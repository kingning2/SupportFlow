# encoding:utf-8
from __future__ import annotations

from channel.channels_control import ChannelsHandler
from channel.console_api import dispatch_console_api


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
