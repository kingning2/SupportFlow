# encoding:utf-8
"""Channel lifecycle (extracted from CowAgent app.py)."""

from __future__ import annotations

import threading
import time

from channel import channel_factory
from common import const
from common.log import logger

_channel_mgr = None
_singleton_cache: dict = {}


def get_channel_manager():
    return _channel_mgr


def set_channel_manager(mgr) -> None:
    global _channel_mgr
    _channel_mgr = mgr


def clear_singleton_cache(channel_name: str | None = None) -> None:
    if channel_name is None:
        _singleton_cache.clear()
        return

    _singleton_cache.pop(channel_name, None)

    cls_map = {
        "web": "channel.web.web_channel.WebChannel",
        "wechatmp": "channel.wechatmp.wechatmp_channel.WechatMPChannel",
        "wechatmp_service": "channel.wechatmp.wechatmp_channel.WechatMPChannel",
        "wechatcom_app": "channel.wechatcom.wechatcomapp_channel.WechatComAppChannel",
        const.FEISHU: "channel.feishu.feishu_channel.FeiShuChanel",
        const.DINGTALK: "channel.dingtalk.dingtalk_channel.DingTalkChanel",
        const.WECOM_BOT: "channel.wecom_bot.wecom_bot_channel.WecomBotChannel",
        const.QQ: "channel.qq.qq_channel.QQChannel",
        const.WEIXIN: "channel.weixin.weixin_channel.WeixinChannel",
        "wx": "channel.wechat.wechat_channel.WechatChannel",
        "wework": "channel.wework.wework_channel.WeworkChannel",
    }
    if channel_name == "wework":
        try:
            from channel.wework.run import reset_wework_client

            reset_wework_client()
        except Exception as e:
            logger.warning(
                "[ChannelManager] Failed to reset wework ntwork client: %s", e
            )

    module_path = cls_map.get(channel_name)
    if not module_path:
        return
    try:
        import importlib

        parts = module_path.rsplit(".", 1)
        module_name, class_name = parts[0], parts[1]
        module = importlib.import_module(module_name)
        wrapper = getattr(module, class_name, None)
        if wrapper and hasattr(wrapper, "__closure__") and wrapper.__closure__:
            for cell in wrapper.__closure__:
                try:
                    cell_contents = cell.cell_contents
                    if isinstance(cell_contents, dict):
                        cell_contents.clear()
                        logger.debug(
                            "[ChannelManager] Cleared singleton cache for %s",
                            class_name,
                        )
                        break
                except ValueError:
                    pass
    except Exception as e:
        logger.warning("[ChannelManager] Failed to clear singleton cache: %s", e)


def parse_channel_type(raw) -> list:
    if isinstance(raw, list):
        return [ch.strip() for ch in raw if ch and str(ch).strip()]
    if isinstance(raw, str):
        return [ch.strip() for ch in raw.split(",") if ch.strip()]
    return []


class ChannelManager:
    def __init__(self):
        self._channels = {}
        self._threads = {}
        self._primary_channel = None
        self._lock = threading.Lock()
        self.cloud_mode = False

    @property
    def channel(self):
        return self._primary_channel

    def get_channel(self, channel_name: str):
        return self._channels.get(channel_name)

    def is_channel_running(self, channel_name: str) -> bool:
        with self._lock:
            thread = self._threads.get(channel_name)
            return thread is not None and thread.is_alive()

    def start(self, channel_names: list, first_start: bool = False):
        with self._lock:
            channels = []
            for name in channel_names:
                if name in ("web", "terminal"):
                    continue
                ch = channel_factory.create_channel(name)
                ch.cloud_mode = self.cloud_mode
                self._channels[name] = ch
                channels.append((name, ch))
                if self._primary_channel is None:
                    self._primary_channel = ch

            if first_start:
                logger.info("[ChannelManager] plugin system disabled in Tauri sidecar")

            for i, (name, ch) in enumerate(channels):
                existing = self._threads.get(name)
                if existing is not None and existing.is_alive():
                    logger.warning(
                        "[ChannelManager] channel '%s' already running, skip duplicate start",
                        name,
                    )
                    continue
                if i > 0:
                    time.sleep(0.1)
                t = threading.Thread(target=self._run_channel, args=(name, ch), daemon=True)
                self._threads[name] = t
                t.start()
                logger.info("[ChannelManager] started '%s'", name)

    def _run_channel(self, name: str, channel):
        try:
            channel.startup()
        except Exception as e:
            logger.error("[ChannelManager] channel '%s' crashed: %s", name, e, exc_info=True)

    def stop(self, channel_name: str):
        with self._lock:
            ch = self._channels.pop(channel_name, None)
            self._threads.pop(channel_name, None)
        if ch and hasattr(ch, "stop"):
            try:
                ch.stop()
            except Exception as e:
                logger.warning("[ChannelManager] stop '%s': %s", channel_name, e)

    def restart(self, channel_name: str):
        self.stop(channel_name)
        clear_singleton_cache(channel_name)
        time.sleep(1)
        self.start([channel_name], first_start=False)
