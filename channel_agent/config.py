# encoding:utf-8
"""Minimal runtime config for the Tauri channel sidecar."""

from __future__ import annotations

import json
import os

from common.log import logger

available_setting = {
    "channel_type": "",
    "group_shared_session": True,
    "group_chat_prefix": ["@bot"],
    "group_chat_keyword": [],
    "group_chat_reply_prefix": "",
    "group_chat_reply_suffix": "",
    "group_at_off": False,
    "single_chat_prefix": [""],
    "single_chat_reply_prefix": "",
    "single_chat_reply_suffix": "",
    "image_create_prefix": [],
    "expires_in_seconds": 3600,
    "speech_recognition": True,
    "group_speech_recognition": False,
    "appdata_dir": ".appdata",
    "agent_workspace": "~/supportflow",
    "wework_exe_path": "",
    "wework_version": "",
    "wework_smart": True,
    "wework_init_wait_seconds": 60,
}

config: dict = {}


def _merge_defaults(raw: dict) -> dict:
    merged = dict(available_setting)
    merged.update(raw)
    return merged


def get_root():
    return os.path.dirname(os.path.abspath(__file__))


def project_config_path():
    override = os.environ.get("CHANNEL_CONFIG_PATH", "").strip()
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


def conf():
    return config


def get_appdata_dir():
    data_path = os.path.join(get_root(), conf().get("appdata_dir", ".appdata"))
    if not os.path.exists(data_path):
        os.makedirs(data_path, exist_ok=True)
    return data_path
