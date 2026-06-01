# encoding:utf-8
"""Logs to stderr as NDJSON; Rust sidecar forwards into tauri-app log."""

import json
import logging
import sys


class _RustForwardHandler(logging.Handler):
    def emit(self, record: logging.LogRecord) -> None:
        try:
            payload = {
                "type": "log",
                "level": record.levelname.lower(),
                "logger": record.name,
                "msg": self.format(record),
            }
            sys.stderr.write(json.dumps(payload, ensure_ascii=False) + "\n")
            sys.stderr.flush()
        except Exception:
            pass


def _get_logger() -> logging.Logger:
    log = logging.getLogger("cowagent.channel")
    if log.handlers:
        return log
    log.setLevel(logging.INFO)
    log.propagate = False
    handler = _RustForwardHandler()
    handler.setFormatter(
        logging.Formatter(
            "[%(levelname)s][%(name)s] %(message)s",
        )
    )
    log.addHandler(handler)
    return log


logger = _get_logger()
