# encoding:utf-8
"""WeCom desktop channel registration."""

from __future__ import annotations

from channel.registry import register_channel, register_extension_rpc


def _clear_wework_singleton_cache() -> None:
    try:
        from channel.wework.run import reset_wework_client

        reset_wework_client()
    except Exception as e:
        from common.log import logger

        logger.warning("[ChannelManager] Failed to reset wework ntwork client: %s", e)

    module_path = "channel.wework.wework_channel.WeworkChannel"
    try:
        import importlib

        module_name, class_name = module_path.rsplit(".", 1)
        module = importlib.import_module(module_name)
        wrapper = getattr(module, class_name, None)
        if wrapper and hasattr(wrapper, "__closure__") and wrapper.__closure__:
            for cell in wrapper.__closure__:
                try:
                    cell_contents = cell.cell_contents
                    if isinstance(cell_contents, dict):
                        cell_contents.clear()
                        from common.log import logger

                        logger.debug(
                            "[ChannelManager] Cleared singleton cache for %s", class_name
                        )
                        break
                except ValueError:
                    pass
    except Exception as e:
        from common.log import logger

        logger.warning("[ChannelManager] Failed to clear singleton cache: %s", e)


def _wework_factory():
    from channel.wework.wework_channel import WeworkChannel

    return WeworkChannel()


def _handle_wework_sync_contacts(_: dict) -> dict:
    from channel import channel_manager

    mgr = channel_manager.get_channel_manager()
    if not mgr:
        return {"status": "error", "message": "wework channel not running"}

    running_ch = mgr.get_channel("wework")
    if not running_ch:
        return {"status": "error", "message": "wework channel not running"}

    started = running_ch.start_contacts_sync(force=True)
    return {"status": "success", "started": started}


def register_wework_channel() -> None:
    register_channel("wework", _wework_factory, cleanup=_clear_wework_singleton_cache)
    register_extension_rpc("wework.sync_contacts", _handle_wework_sync_contacts)


register_wework_channel()
