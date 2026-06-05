# encoding:utf-8
from __future__ import annotations

from channel import channel_manager


def handle_rust_request(req: dict) -> dict:
    req_id = req.get("id")
    method = req.get("method")
    params = req.get("params") or {}
    try:
        def handle_channel_start(p: dict) -> dict:
            channel_name = p.get("channel")
            if not channel_name:
                return {"status": "error", "message": "channel required"}

            mgr = channel_manager.get_channel_manager()
            if mgr is None:
                from channel.stdio_server import _start_configured_channels

                _start_configured_channels(first_start=False)
            elif not mgr.is_channel_running(channel_name):
                mgr.start([channel_name], first_start=False)

            return {"status": "success", "channel": channel_name}

        def handle_channel_stop(p: dict) -> dict:
            channel_name = p.get("channel")
            if not channel_name:
                return {"status": "error", "message": "channel required"}

            mgr = channel_manager.get_channel_manager()
            if mgr:
                mgr.stop(channel_name)
                channel_manager.clear_singleton_cache(channel_name)

            return {"status": "success", "channel": channel_name}

        def handle_channel_restart(p: dict) -> dict:
            channel_name = p.get("channel")
            if not channel_name:
                return {"status": "error", "message": "channel required"}

            mgr = channel_manager.get_channel_manager()
            if mgr is None:
                from channel.stdio_server import _start_configured_channels

                _start_configured_channels(first_start=False)
            else:
                mgr.restart(channel_name)

            return {"status": "success", "channel": channel_name}

        def handle_wework_sync_contacts(_: dict) -> dict:
            mgr = channel_manager.get_channel_manager()
            if not mgr:
                return {"status": "error", "message": "wework channel not running"}

            running_ch = mgr.get_channel("wework")
            if not running_ch:
                return {"status": "error", "message": "wework channel not running"}

            started = running_ch.start_contacts_sync(force=True)
            return {"status": "success", "started": started}

        def handle_ping(_: dict) -> dict:
            return {"status": "success", "pong": True}

        handlers = {
            "channel.start": handle_channel_start,
            "channel.stop": handle_channel_stop,
            "channel.restart": handle_channel_restart,
            "wework.sync_contacts": handle_wework_sync_contacts,
            "ping": handle_ping,
        }

        handler = handlers.get(method)
        if handler is None:
            return {"id": req_id, "error": f"unknown method: {method}"}

        result = handler(params)
        if result.get("status") != "success":
            return {
                "id": req_id,
                "error": result.get("message", "request failed"),
                "result": result,
            }
        return {"id": req_id, "result": result}
    except Exception as e:
        return {"id": req_id, "error": str(e)}
