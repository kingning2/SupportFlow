# encoding:utf-8
"""Channel-only config module for Tauri sidecar."""

from __future__ import annotations

import json
import os

from common.log import logger

available_setting = {
    "agent": True,
    "channel_type": "",
    "group_shared_session": True,
    "group_name_white_list": [],
    "group_name_keyword_white_list": [],
    "group_chat_in_one_session": [],
    "group_chat_prefix": ["@bot"],
    "group_chat_keyword": [],
    "group_chat_reply_prefix": "",
    "group_chat_reply_suffix": "",
    "group_at_off": False,
    "single_chat_prefix": [""],
    "single_chat_reply_prefix": "",
    "single_chat_reply_suffix": "",
    "nick_name_black_list": [],
    "trigger_by_self": False,
    "image_create_prefix": [],
    "concurrency_in_session": 1,
    "expires_in_seconds": 3600,
    "speech_recognition": True,
    "group_speech_recognition": False,
    "voice_reply_voice": False,
    "always_reply_voice": False,
    "subscribe_msg": "",
    "appdata_dir": ".appdata",
    "agent_workspace": "~/cow",
    "use_proxy": False,
    "proxy": "",
    "hot_reload": False,
    "web_console": False,
    "web_port": 9899,
    "weixin_token": "",
    "weixin_base_url": "https://ilinkai.weixin.qq.com",
    "weixin_cdn_base_url": "https://novac2c.cdn.weixin.qq.com/c2c",
    "weixin_credentials_path": "~/.weixin_cow_credentials.json",
    "wework_exe_path": "",
    "wework_version": "",
    "wework_smart": True,
    "wework_init_wait_seconds": 60,
    "feishu_port": 9891,
    "feishu_app_id": "",
    "feishu_app_secret": "",
    "feishu_token": "",
    "feishu_event_mode": "websocket",
    "feishu_stream_reply": True,
    "dingtalk_client_id": "",
    "dingtalk_client_secret": "",
    "dingtalk_robot_code": "",
    "dingtalk_card_enabled": False,
    "wecom_bot_id": "",
    "wecom_bot_secret": "",
    "wechatmp_token": "",
    "wechatmp_port": 8080,
    "wechatmp_app_id": "",
    "wechatmp_app_secret": "",
    "wechatmp_aes_key": "",
    "wechatcom_corp_id": "",
    "wechatcomapp_token": "",
    "wechatcomapp_port": 9898,
    "wechatcomapp_secret": "",
    "wechatcomapp_agent_id": "",
    "wechatcomapp_aes_key": "",
    "qq_app_id": "",
    "qq_app_secret": "",
}

config: dict = {}


def _merge_defaults(raw: dict) -> dict:
    merged = dict(available_setting)
    merged.update(raw)
    return merged


def _mask_sensitive(values: dict) -> dict:
    masked = dict(values)
    for key, value in list(masked.items()):
        if not isinstance(value, str):
            continue
        if ("key" in key or "secret" in key or "token" in key) and len(value) >= 8:
            masked[key] = value[:3] + "*" * 5 + value[-3:]
    return masked


def get_root():
    return os.path.dirname(os.path.abspath(__file__))


def project_config_path():
    override = os.environ.get("COW_CONFIG_PATH", "").strip()
    if override:
        return os.path.abspath(override)
    return os.path.join(get_root(), "config.json")


def read_file(path):
    with open(path, mode="r", encoding="utf-8-sig") as f:
        return f.read()


def load_config():
    global config
    config_path = project_config_path()
    if not os.path.exists(config_path):
        logger.warning("[INIT] config not found: %s", config_path)
        config = dict(available_setting)
        return
    config_str = read_file(config_path)
    raw = json.loads(config_str)
    if not isinstance(raw, dict):
        raise ValueError("config must be a JSON object")
    config = _merge_defaults(raw)
    logger.info("[INIT] load channel config: %s", _mask_sensitive(config))


def conf():
    return config


def get_appdata_dir():
    data_path = os.path.join(get_root(), conf().get("appdata_dir", ".appdata"))
    if not os.path.exists(data_path):
        os.makedirs(data_path, exist_ok=True)
    return data_path


def subscribe_msg():
    trigger_prefix = conf().get("single_chat_prefix", [""])[0]
    msg = conf().get("subscribe_msg", "")
    return msg.format(trigger_prefix=trigger_prefix)
