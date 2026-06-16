# encoding:utf-8
"""Channel lifecycle for the supported desktop channels.

This module is part of the Python sidecar compatibility layer. It may keep the
minimum SDK-specific lifecycle needed to boot and stop channel adapters inside
the sidecar process, but it must not grow into a desktop application
orchestration layer again.

Allowed here:
- sidecar-local channel instantiation
- SDK startup / shutdown glue
- sidecar thread ownership for channel adapters

Should move to Rust instead:
- desktop runtime policy
- cross-Webview state decisions
- restart / retry strategy owned by the app
- user-visible orchestration and persistence rules
"""

from __future__ import annotations

import threading
import time

from common.log import logger

from channel.registry import clear_channel_cache, create_channel

_channel_mgr = None


def get_channel_manager():
    """获取当前激活的通道管理器单例。"""
    return _channel_mgr


def set_channel_manager(mgr) -> None:
    """设置当前激活的通道管理器单例。"""
    global _channel_mgr
    _channel_mgr = mgr


def clear_singleton_cache(channel_name: str | None = None) -> None:
    """清理指定通道或全部通道的单例缓存。"""
    clear_channel_cache(channel_name)


def parse_channel_type(raw) -> list:
    """解析 channel_type 并在保持顺序的前提下去重。"""
    if isinstance(raw, list):
        items = [str(ch).strip() for ch in raw if ch and str(ch).strip()]
    elif isinstance(raw, str):
        items = [ch.strip() for ch in raw.split(",") if ch.strip()]
    else:
        return []
    seen: set[str] = set()
    out: list[str] = []
    for name in items:
        if name in seen:
            continue
        seen.add(name)
        out.append(name)
    return out


def parse_external_channel_type(raw) -> list:
    """解析桌面侧可启动的外部通道列表。"""
    return parse_channel_type(raw)


class ChannelManager:
    """管理 sidecar 中的通道实例与工作线程。"""

    def __init__(self):
        """初始化空的运行时状态。"""
        self._channels = {}
        self._threads = {}
        self._primary_channel = None
        self._lock = threading.Lock()
        self.cloud_mode = False

    @property
    def channel(self):
        """返回首个启动的通道实例。"""
        return self._primary_channel

    def get_channel(self, channel_name: str):
        """按名称返回当前运行中的通道实例。"""
        return self._channels.get(channel_name)

    def is_channel_running(self, channel_name: str) -> bool:
        """判断指定通道线程是否仍然存活。"""
        with self._lock:
            thread = self._threads.get(channel_name)
            return thread is not None and thread.is_alive()

    def start(self, channel_names: list, first_start: bool = False):
        """启动指定通道列表中尚未运行的通道。"""
        channel_names = parse_external_channel_type(channel_names)
        with self._lock:
            channels = []
            for name in channel_names:
                ch = create_channel(name)
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
                thread = threading.Thread(
                    target=self._run_channel,
                    args=(name, ch),
                    daemon=True,
                )
                self._threads[name] = thread
                thread.start()
                logger.info("[ChannelManager] started '%s'", name)

    def _run_channel(self, name: str, channel):
        """在线程中执行单个通道的启动流程。"""
        try:
            channel.startup()
        except Exception as e:
            logger.error("[ChannelManager] channel '%s' crashed: %s", name, e, exc_info=True)

    def stop(self, channel_name: str):
        """停止指定通道。"""
        with self._lock:
            ch = self._channels.pop(channel_name, None)
            self._threads.pop(channel_name, None)
        if ch and hasattr(ch, "stop"):
            try:
                ch.stop()
            except Exception as e:
                logger.warning("[ChannelManager] stop '%s': %s", channel_name, e)

    def restart(self, channel_name: str):
        """重启指定通道，并同步清理其单例缓存。"""
        self.stop(channel_name)
        clear_singleton_cache(channel_name)
        time.sleep(1)
        self.start([channel_name], first_start=False)
