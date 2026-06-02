# encoding:utf-8
"""Channel catalog and connect/disconnect/save (extracted from web console)."""

from __future__ import annotations

import json
import os
import sys
import threading
import time
from collections import OrderedDict

from channel import channel_manager
from channel.channel_manager import clear_singleton_cache, parse_channel_type
from common.log import logger
from config import conf, project_config_path


_WEWORK_RESTART_KEYS = frozenset(
    {"wework_exe_path", "wework_version", "wework_smart", "wework_init_wait_seconds"}
)

_WEWORK_CONNECT_DEFAULTS = {
    "wework_version": "4.0.8.6027",
    "wework_init_wait_seconds": 60,
    "wework_smart": True,
}
_WEWORK_DEFAULT_VERSION = "4.0.8.6027"
_WEWORK_DEFAULT_INIT_WAIT = 60


def _wework_session_running() -> bool:
    try:
        from channel.wework.run import wework_session_ready

        mgr = channel_manager.get_channel_manager()
        ch = mgr.get_channel("wework") if mgr else None
        return bool(
            mgr
            and mgr.is_channel_running("wework")
            and ch
            and getattr(ch, "_channel_ready", False)
            and wework_session_ready()
        )
    except Exception:
        return False


class ChannelsHandler:
    """Manage external channel configurations for Tauri / web console."""

    CHANNEL_DEFS = OrderedDict(
        [
            (
                "weixin",
                {
                    "label": {"zh": "微信(官方)", "en": "WeChat (Official)"},
                    "icon": "fa-comment",
                    "color": "emerald",
                    "fields": [],
                    "hint": {
                        "zh": "微信官方 ilink Bot，扫码后新增机器人助理；不支持群聊。",
                        "en": "Official WeChat ilink bot; QR login; no group chat.",
                    },
                },
            ),
            (
                "wx",
                {
                    "label": {"zh": "个人微信(itchat)", "en": "WeChat (itchat)"},
                    "icon": "fa-brands fa-weixin",
                    "color": "green",
                    "fields": [
                        {
                            "key": "hot_reload",
                            "label": {"zh": "热重载登录", "en": "Hot reload login"},
                            "type": "bool",
                            "default": False,
                        },
                    ],
                    "hint": {
                        "zh": "基于 itchat 网页协议，存在封号风险，仅建议测试。支持私聊与群聊；登录态保存在数据目录 itchat.pkl。",
                        "en": "itchat web protocol; account risk; test use only. Supports DM and groups.",
                    },
                },
            ),
            (
                "feishu",
                {
                    "label": {"zh": "飞书", "en": "Feishu"},
                    "icon": "fa-paper-plane",
                    "color": "blue",
                    "fields": [
                        {"key": "feishu_app_id", "label": "App ID", "type": "text"},
                        {"key": "feishu_app_secret", "label": "App Secret", "type": "secret"},
                    ],
                },
            ),
            (
                "dingtalk",
                {
                    "label": {"zh": "钉钉", "en": "DingTalk"},
                    "icon": "fa-comments",
                    "color": "blue",
                    "fields": [
                        {"key": "dingtalk_client_id", "label": "Client ID", "type": "text"},
                        {"key": "dingtalk_client_secret", "label": "Client Secret", "type": "secret"},
                    ],
                },
            ),
            (
                "wecom_bot",
                {
                    "label": {"zh": "企微智能机器人", "en": "WeCom Bot"},
                    "icon": "fa-robot",
                    "color": "emerald",
                    "fields": [
                        {"key": "wecom_bot_id", "label": "Bot ID", "type": "text"},
                        {"key": "wecom_bot_secret", "label": "Secret", "type": "secret"},
                    ],
                },
            ),
            (
                "wework",
                {
                    "label": {"zh": "企微个人号", "en": "WeCom Desktop"},
                    "icon": "fa-desktop",
                    "color": "emerald",
                    "fields": [
                        {
                            "key": "wework_exe_path",
                            "label": {"zh": "企微程序路径", "en": "WeCom executable path"},
                            "type": "text",
                            "placeholder": {
                                "zh": "如 D:\\WXWork\\4.0.8.6027\\WXWork.exe",
                                "en": "e.g. D:\\WXWork\\4.0.8.6027\\WXWork.exe",
                            },
                        },
                    ],
                    "hint": {
                        "zh": "需 Windows 与企业微信 PC 4.0.8.6027；仅当本机有多个安装版本时再改路径。",
                        "en": "Windows + WeCom 4.0.8.6027; change path only if multiple installs.",
                    },
                },
            ),
            (
                "qq",
                {
                    "label": {"zh": "QQ 机器人", "en": "QQ Bot"},
                    "icon": "fa-comment",
                    "color": "blue",
                    "fields": [
                        {"key": "qq_app_id", "label": "App ID", "type": "text"},
                        {"key": "qq_app_secret", "label": "App Secret", "type": "secret"},
                    ],
                },
            ),
            (
                "wechatcom_app",
                {
                    "label": {"zh": "企微自建应用", "en": "WeCom App"},
                    "icon": "fa-building",
                    "color": "emerald",
                    "fields": [
                        {"key": "wechatcom_corp_id", "label": "Corp ID", "type": "text"},
                        {"key": "wechatcomapp_agent_id", "label": "Agent ID", "type": "text"},
                        {"key": "wechatcomapp_secret", "label": "Secret", "type": "secret"},
                        {"key": "wechatcomapp_token", "label": "Token", "type": "secret"},
                        {"key": "wechatcomapp_aes_key", "label": "AES Key", "type": "secret"},
                        {"key": "wechatcomapp_port", "label": "Port", "type": "number", "default": 9898},
                    ],
                },
            ),
            (
                "wechatmp",
                {
                    "label": {"zh": "公众号", "en": "WeChat MP"},
                    "icon": "fa-comment-dots",
                    "color": "emerald",
                    "fields": [
                        {"key": "wechatmp_app_id", "label": "App ID", "type": "text"},
                        {"key": "wechatmp_app_secret", "label": "App Secret", "type": "secret"},
                        {"key": "wechatmp_token", "label": "Token", "type": "secret"},
                        {"key": "wechatmp_aes_key", "label": "AES Key", "type": "secret"},
                        {"key": "wechatmp_port", "label": "Port", "type": "number", "default": 8080},
                    ],
                },
            ),
        ]
    )

    @staticmethod
    def _get_channel_login_status(channel_name: str) -> str:
        try:
            mgr = channel_manager.get_channel_manager()
            if mgr:
                ch = mgr.get_channel(channel_name)
                if ch and hasattr(ch, "login_status"):
                    return ch.login_status
        except Exception:
            pass
        return "unknown"

    @staticmethod
    def _get_wework_login_profile() -> dict | None:
        try:
            mgr = channel_manager.get_channel_manager()
            if mgr:
                ch = mgr.get_channel("wework")
                if ch and getattr(ch, "user_id", None):
                    display = getattr(ch, "name", "") or ""
                    return {
                        "user_id": str(ch.user_id),
                        "display_name": display,
                    }
            from channel.wework.run import get_wework, wework_session_ready

            if wework_session_ready():
                client = get_wework()
                if client:
                    info = client.get_login_info() or {}
                    user_id = info.get("user_id")
                    if user_id:
                        display = info.get("nickname") or info.get("username") or ""
                        return {
                            "user_id": str(user_id),
                            "display_name": str(display),
                        }
        except Exception:
            pass
        return None

    @staticmethod
    def _mask_secret(value: str) -> str:
        if not value or len(value) <= 8:
            return value
        return value[:4] + "*" * (len(value) - 8) + value[-4:]

    @staticmethod
    def _parse_channel_list(raw) -> list:
        return parse_channel_type(raw)

    @classmethod
    def _active_channel_set(cls) -> set:
        return set(cls._parse_channel_list(conf().get("channel_type", "")))

    @classmethod
    def _runtime_channel_active(cls, ch_name: str, in_config: bool) -> bool:
        """Runtime IM channels: active = thread running (and wework session ready)."""
        runtime_names = {"wework", "weixin", "wx"}
        if ch_name not in runtime_names:
            return in_config
        if not in_config:
            return False
        mgr = channel_manager.get_channel_manager()
        if not mgr or not mgr.is_channel_running(ch_name):
            return False
        if ch_name == "wework":
            ch = mgr.get_channel("wework") if mgr else None
            return bool(ch and getattr(ch, "_channel_ready", False))
        return True

    def list_channels_dict(self) -> dict:
        local_config = conf()
        active_channels = self._active_channel_set()
        channels = []
        for ch_name, ch_def in self.CHANNEL_DEFS.items():
            fields_out = []
            for f in ch_def["fields"]:
                raw_val = local_config.get(f["key"], f.get("default", ""))
                if f["type"] == "secret" and raw_val:
                    display_val = self._mask_secret(str(raw_val))
                else:
                    display_val = raw_val
                fields_out.append(
                    {
                        "key": f["key"],
                        "label": f["label"],
                        "type": f["type"],
                        "value": display_val,
                        "default": f.get("default", ""),
                    }
                )
            in_config = ch_name in active_channels
            ch_info = {
                "name": ch_name,
                "label": ch_def["label"],
                "icon": ch_def["icon"],
                "color": ch_def["color"],
                "active": self._runtime_channel_active(ch_name, in_config),
                "fields": fields_out,
            }
            if ch_name in ("weixin", "wx", "wework") and in_config:
                ch_info["login_status"] = self._get_channel_login_status(ch_name)
            if ch_name == "wework":
                profile = self._get_wework_login_profile()
                if profile:
                    ch_info["login_profile"] = profile
            if "hint" in ch_def:
                ch_info["hint"] = ch_def["hint"]
            channels.append(ch_info)
        return {"status": "success", "channels": channels}

    def dispatch_action(self, action: str, channel_name: str, config: dict | None = None) -> dict:
        config = config or {}
        if action == "save":
            return self._handle_save(channel_name, config)
        if action == "connect":
            return self._handle_connect(channel_name, config)
        if action == "disconnect":
            return self._handle_disconnect(channel_name)
        return {"status": "error", "message": f"unknown action: {action}"}

    def _handle_save(self, channel_name: str, updates: dict):
        ch_def = self.CHANNEL_DEFS[channel_name]
        valid_keys = {f["key"] for f in ch_def["fields"]}
        secret_keys = {f["key"] for f in ch_def["fields"] if f["type"] == "secret"}

        local_config = conf()
        applied = {}
        for key, value in updates.items():
            if key not in valid_keys:
                continue
            if key in secret_keys:
                if not value or (len(value) > 8 and "*" * 4 in value):
                    continue
            field_def = next((f for f in ch_def["fields"] if f["key"] == key), None)
            if field_def:
                if field_def["type"] == "number":
                    value = int(value)
                elif field_def["type"] in ("bool", "checkbox"):
                    value = bool(value)
            local_config[key] = value
            applied[key] = value

        if not applied:
            return {"status": "error", "message": "no valid fields to update"}

        self._persist_config_patch(applied)
        logger.info("[Channels] '%s' config updated: %s", channel_name, list(applied.keys()))
        should_restart = channel_name in self._active_channel_set()
        if (
            channel_name == "wework"
            and should_restart
            and _wework_session_running()
            and not (_WEWORK_RESTART_KEYS & set(applied.keys()))
        ):
            logger.info(
                "[Channels] wework already logged in; skip restart for non-exe config patch"
            )
            should_restart = False
        if should_restart:
            mgr = channel_manager.get_channel_manager()
            if mgr:
                threading.Thread(target=mgr.restart, args=(channel_name,), daemon=True).start()
        return {
            "status": "success",
            "applied": list(applied.keys()),
            "restarted": should_restart,
        }

    def _handle_connect(self, channel_name: str, updates: dict):
        ch_def = self.CHANNEL_DEFS[channel_name]
        valid_keys = {f["key"] for f in ch_def["fields"]}
        secret_keys = {f["key"] for f in ch_def["fields"] if f["type"] == "secret"}

        if channel_name == "feishu":
            updates.setdefault("feishu_event_mode", "websocket")
            valid_keys.add("feishu_event_mode")

        if channel_name == "wework":
            for key, default in _WEWORK_CONNECT_DEFAULTS.items():
                updates.setdefault(key, default)
            valid_keys |= set(_WEWORK_CONNECT_DEFAULTS.keys())

        local_config = conf()
        applied = {}
        for key, value in updates.items():
            if key not in valid_keys:
                continue
            if key in secret_keys:
                if not value or (len(value) > 8 and "*" * 4 in value):
                    continue
            field_def = next((f for f in ch_def["fields"] if f["key"] == key), None)
            if field_def:
                if field_def["type"] == "number":
                    value = int(value)
                elif field_def["type"] in ("bool", "checkbox"):
                    value = bool(value)
            local_config[key] = value
            applied[key] = value

        for f in ch_def["fields"]:
            if f["key"] not in applied and f["key"] not in updates:
                default = f.get("default")
                if default is not None:
                    local_config[f["key"]] = default
                    applied[f["key"]] = default

        existing = self._parse_channel_list(conf().get("channel_type", ""))
        if channel_name not in existing:
            existing.append(channel_name)
        new_channel_type = ",".join(existing)
        local_config["channel_type"] = new_channel_type

        self._persist_config_patch({**applied, "channel_type": new_channel_type})
        logger.info("[Channels] '%s' connecting, channel_type=%s", channel_name, new_channel_type)

        if (
            channel_name == "wework"
            and _wework_session_running()
            and not (_WEWORK_RESTART_KEYS & set(applied.keys()))
        ):
            logger.info("[Channels] wework already running; skip reconnect (no relogin)")
            return {
                "status": "success",
                "channel_type": new_channel_type,
                "already_connected": True,
            }

        def _do_start():
            try:
                mgr = channel_manager.get_channel_manager()
                if mgr is None:
                    from channel.stdio_server import _start_configured_channels

                    logger.info(
                        "[Channels] ChannelManager missing; starting configured channels"
                    )
                    _start_configured_channels(first_start=False)
                    return

                restart_keys = _WEWORK_RESTART_KEYS & set(applied.keys())
                if channel_name == "wework":
                    from channel.wework.run import (
                        wework_desktop_process_running,
                        wework_session_ready,
                    )

                    if (
                        mgr.is_channel_running(channel_name)
                        and wework_session_ready()
                        and not restart_keys
                    ):
                        logger.info(
                            "[Channels] wework already running with session; skip start"
                        )
                        return
                    if mgr.is_channel_running(channel_name) and not restart_keys:
                        logger.info(
                            "[Channels] wework channel thread already active; skip duplicate start"
                        )
                        return
                    if wework_desktop_process_running() and not restart_keys:
                        logger.info(
                            "[Channels] WeCom desktop already up; start channel thread only"
                        )
                        mgr.start([channel_name], first_start=False)
                        return
                    if mgr.is_channel_running(channel_name):
                        mgr.stop(channel_name)
                        time.sleep(2)
                    if restart_keys:
                        clear_singleton_cache(channel_name)
                    mgr.start([channel_name], first_start=False)
                    return

                existing_ch = mgr.get_channel(channel_name)
                if existing_ch is not None:
                    mgr.stop(channel_name)
                time.sleep(2)
                clear_singleton_cache(channel_name)
                mgr.start([channel_name], first_start=False)
            except Exception as e:
                logger.error("[Channels] start '%s' failed: %s", channel_name, e, exc_info=True)

        threading.Thread(target=_do_start, daemon=True).start()
        return {"status": "success", "channel_type": new_channel_type}

    def _handle_disconnect(self, channel_name: str):
        existing = self._parse_channel_list(conf().get("channel_type", ""))
        existing = [ch for ch in existing if ch != channel_name]
        new_channel_type = ",".join(existing)

        conf()["channel_type"] = new_channel_type
        self._persist_config_patch({"channel_type": new_channel_type})

        def _do_stop():
            try:
                mgr = channel_manager.get_channel_manager()
                if mgr:
                    mgr.stop(channel_name)
                clear_singleton_cache(channel_name)
            except Exception as e:
                logger.warning("[Channels] stop '%s': %s", channel_name, e)

        threading.Thread(target=_do_stop, daemon=True).start()
        return {"status": "success", "channel_type": new_channel_type}

    @staticmethod
    def _persist_config_patch(patch: dict) -> None:
        config_path = project_config_path()
        file_cfg = {}
        if os.path.isfile(config_path):
            with open(config_path, "r", encoding="utf-8") as f:
                file_cfg = json.load(f)
        file_cfg.update(patch)
        with open(config_path, "w", encoding="utf-8") as f:
            json.dump(file_cfg, f, indent=4, ensure_ascii=False)
        conf().update(patch)
