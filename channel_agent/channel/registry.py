# encoding:utf-8
"""Channel adapter registry — register factories to add new desktop channels."""

from __future__ import annotations

from collections.abc import Callable
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from channel.adapter import ChannelAdapter

Factory = Callable[[], "ChannelAdapter"]
CleanupHook = Callable[[], None]

_REGISTRY: dict[str, Factory] = {}
_CLEANUP: dict[str, CleanupHook] = {}
_EXTENSION_RPC: dict[str, Callable[[dict], dict]] = {}


def register_channel(
    channel_type: str,
    factory: Factory,
    *,
    cleanup: CleanupHook | None = None,
) -> None:
    """Register a channel adapter factory for sidecar instantiation."""
    _REGISTRY[channel_type] = factory
    if cleanup is not None:
        _CLEANUP[channel_type] = cleanup


def register_extension_rpc(method: str, handler: Callable[[dict], dict]) -> None:
    """Register a channel-specific Rust→Python RPC handler (e.g. wework.sync_contacts)."""
    _EXTENSION_RPC[method] = handler


def known_channel_types() -> list[str]:
    return list(_REGISTRY.keys())


def create_channel(channel_type: str) -> "ChannelAdapter":
    factory = _REGISTRY.get(channel_type)
    if factory is None:
        raise RuntimeError(f"unknown channel type: {channel_type!r}")
    ch = factory()
    ch.channel_type = channel_type
    return ch


def clear_channel_cache(channel_name: str | None = None) -> None:
    if channel_name is None:
        for hook in _CLEANUP.values():
            try:
                hook()
            except Exception:
                pass
        return
    hook = _CLEANUP.get(channel_name)
    if hook is not None:
        hook()


def extension_rpc_handler(method: str) -> Callable[[dict], dict] | None:
    return _EXTENSION_RPC.get(method)
