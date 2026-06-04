# encoding:utf-8
"""Bidirectional NDJSON RPC with Rust (Python-initiated requests on stdout)."""

from __future__ import annotations

import itertools
import json
import sys
import threading
from typing import Any

_lock = threading.Lock()
_pending: dict[int, tuple[threading.Event, dict | None]] = {}
_id_gen = itertools.count(10_000)
_reader_started = False


def _ensure_reader() -> None:
    global _reader_started
    if _reader_started:
        return
    _reader_started = True
    t = threading.Thread(target=_stdin_loop, name="rust-ipc-reader", daemon=True)
    t.start()


def _stdin_loop() -> None:
    from channel.rpc_handlers import handle_rust_request

    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            msg = json.loads(line)
        except json.JSONDecodeError:
            continue
        if msg.get("method"):
            resp = handle_rust_request(msg)
            sys.stdout.write(json.dumps(resp, ensure_ascii=False) + "\n")
            sys.stdout.flush()
            continue
        req_id = msg.get("id")
        if req_id is None:
            continue
        with _lock:
            entry = _pending.pop(int(req_id), None)
        if entry:
            _event, holder = entry
            holder["msg"] = msg
            _event.set()


def notify_rust(method: str, params: dict | None = None) -> None:
    """Fire-and-forget notification to Rust (no response expected)."""
    _ensure_reader()
    payload = {"method": method, "params": params or {}}
    with _lock:
        sys.stdout.write(json.dumps(payload, ensure_ascii=False) + "\n")
        sys.stdout.flush()


def call_rust(method: str, params: dict | None = None, timeout: float = 300.0) -> dict:
    _ensure_reader()
    params = params or {}
    with _lock:
        req_id = next(_id_gen)
        event = threading.Event()
        holder: dict[str, Any] = {}
        _pending[req_id] = (event, holder)

    payload = {"id": req_id, "method": method, "params": params}
    with _lock:
        sys.stdout.write(json.dumps(payload, ensure_ascii=False) + "\n")
        sys.stdout.flush()

    if not event.wait(timeout=timeout):
        with _lock:
            _pending.pop(req_id, None)
        raise TimeoutError(f"Rust RPC timeout: {method}")

    msg = holder.get("msg") or {}
    if msg.get("error"):
        raise RuntimeError(str(msg["error"]))
    result = msg.get("result")
    if not isinstance(result, dict):
        raise RuntimeError(f"invalid Rust RPC result for {method}")
    return result


def call_rust_bool(method: str, params: dict | None = None, timeout: float = 300.0) -> bool:
    """Call Rust RPC and read a boolean `value` field."""
    result = call_rust(method, params, timeout=timeout)
    value = result.get("value")
    if isinstance(value, bool):
        return value
    raise RuntimeError(f"invalid Rust RPC bool result for {method}")
