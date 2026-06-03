# encoding:utf-8
from __future__ import annotations

import os

from channel.channels_control import ChannelsHandler
from channel.console_api import dispatch_console_api
from common.log import logger


def handle_rust_request(req: dict) -> dict:
    req_id = req.get("id")
    method = req.get("method")
    params = req.get("params") or {}
    try:
        if method == "channels.list":
            result = ChannelsHandler().list_channels_dict()
        elif method == "channels.action":
            handler = ChannelsHandler()
            action = params.get("action")
            channel_name = params.get("channel")
            if not action or not channel_name:
                result = {"status": "error", "message": "action and channel required"}
            elif channel_name not in handler.CHANNEL_DEFS:
                result = {"status": "error", "message": f"unknown channel: {channel_name}"}
            else:
                result = handler.dispatch_action(action, channel_name, params.get("config", {}))
        elif method == "channels.status":
            from channel import channel_manager
            from channel.channel_manager import parse_external_channel_type
            from config import conf

            mgr = channel_manager.get_channel_manager()
            configured = parse_external_channel_type(conf().get("channel_type", ""))
            running = []
            if mgr:
                for name in configured:
                    if mgr.is_channel_running(name):
                        running.append(name)
            result = {
                "status": "success",
                "configured": configured,
                "running": running,
                "manager_ready": mgr is not None,
            }
        elif method == "channels.autostart":
            from channel.stdio_server import _start_configured_channels
            from channel import channel_manager
            from channel.channel_manager import parse_external_channel_type
            from config import conf

            mgr = channel_manager.get_channel_manager()
            configured = parse_external_channel_type(conf().get("channel_type", ""))
            dev_channel = (os.environ.get("DEV_CHANNEL") or "").strip()
            # Standalone desktop apps (wework / wx): connect only from UI, not on boot.
            if dev_channel in ("wework", "wx"):
                logger.info(
                    "[ChannelStdio] autostart skipped for DEV_CHANNEL=%s (manual connect)",
                    dev_channel,
                )
                configured = []
            elif dev_channel:
                configured = [n for n in configured if n == dev_channel]
            if mgr is None:
                _start_configured_channels(first_start=False)
            elif configured:
                missing = [n for n in configured if not mgr.is_channel_running(n)]
                if missing:
                    logger.info("[ChannelStdio] autostart missing channels: %s", missing)
                    mgr.start(missing, first_start=False)
            result = {"status": "success", "autostart": True}
        elif method == "ping":
            result = {"status": "success", "pong": True}
        elif method == "console.api":
            result = dispatch_console_api(
                params.get("path", ""),
                params.get("method", "GET"),
                params.get("body") if isinstance(params.get("body"), dict) else {},
            )
        else:
            return {"id": req_id, "error": f"unknown method: {method}"}
        if result.get("status") != "success":
            return {
                "id": req_id,
                "error": result.get("message", "request failed"),
                "result": result,
            }
        return {"id": req_id, "result": result}
    except Exception as e:
        return {"id": req_id, "error": str(e)}
