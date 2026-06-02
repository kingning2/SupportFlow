# encoding:utf-8
"""Tauri channel sidecar: NDJSON RPC + ChannelManager runtime."""

from __future__ import annotations

import os
import signal
import sys
import threading
import time

from channel import channel_manager
from channel.channel_manager import ChannelManager, parse_channel_type
from channel.rpc_handlers import handle_rust_request
from channel.rust_ipc import _ensure_reader
from common.log import logger
from config import conf, load_config


def _start_configured_channels(first_start: bool) -> None:
    raw = conf().get("channel_type", "")
    names = parse_channel_type(raw)
    names = [n for n in names if n not in ("web", "terminal")]
    mgr = ChannelManager()
    channel_manager.set_channel_manager(mgr)
    if names:
        logger.info("[ChannelStdio] starting channels: %s", ", ".join(names))
        mgr.start(names, first_start=first_start)
    else:
        logger.info("[ChannelStdio] no external channels in config")


def _force_utf8_stdio() -> None:
    """Frozen exe on Windows may ignore PYTHONIOENCODING; keep NDJSON on stdout valid UTF-8."""
    import io

    for name in ("stdout", "stderr"):
        stream = getattr(sys, name, None)
        buf = getattr(stream, "buffer", None)
        if buf is None:
            continue
        setattr(
            sys,
            name,
            io.TextIOWrapper(buf, encoding="utf-8", errors="replace", line_buffering=True),
        )


def run_stdio_server() -> None:
    os.environ.setdefault("TAURI_CHANNEL_MODE", "1")
    _force_utf8_stdio()
    load_config()
    _ensure_reader()

    conf()["web_console"] = False
    conf()["agent"] = conf().get("agent", True)

    def _sig(_signum, _frame):
        logger.info("[ChannelStdio] shutdown signal")
        sys.exit(0)

    signal.signal(signal.SIGINT, _sig)
    if hasattr(signal, "SIGTERM"):
        signal.signal(signal.SIGTERM, _sig)

    # Tauri desktop calls `channels.autostart` over RPC after sidecar is up.
    # Built-in deferred autostart here would start the same channels twice
    # (each wework thread calls ntwork open() → multiple WeCom processes).
    if os.environ.get("TAURI_CHANNEL_MODE") != "1":
        def _deferred_autostart() -> None:
            delay = float(os.environ.get("CHANNEL_START_DELAY_SECS", "1"))
            if delay > 0:
                time.sleep(delay)
            _start_configured_channels(first_start=True)

        threading.Thread(
            target=_deferred_autostart,
            name="channel-autostart",
            daemon=True,
        ).start()
    # else:
        # logger.info(
        #     "[ChannelStdio] TAURI_CHANNEL_MODE=1: channel autostart deferred to Rust RPC"
        # )

    while True:
        time.sleep(3600)
