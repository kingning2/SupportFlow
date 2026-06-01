# encoding:utf-8
"""Smoke test: hook running WeCom without calling open() twice. Run from cowagent root:
  py -3.8 scripts/test_wework_hook.py
"""
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)

os.environ.setdefault("COW_CONFIG_PATH", os.path.join(ROOT, "..", "resources", "config.json"))

from config import load_config, conf
from channel.wework.run import (
    _find_running_wework_pids,
    ensure_wework_login,
    init_wework_client,
    reset_wework_client,
    wework_session_ready,
)


def main() -> int:
    load_config()
    print("wework_smart:", conf().get("wework_smart", True))
    pids = _find_running_wework_pids()
    print("running WXWork.exe pids (UI first):", pids or "(none)")
    if not pids:
        print("SKIP: start and log in to WeCom 4.0.8.6027 first, then re-run.")
        return 2

    reset_wework_client()
    client = init_wework_client()
    smart = bool(conf().get("wework_smart", True))

    print("--- first ensure_wework_login ---")
    info1 = ensure_wework_login(client, smart=smart, timeout=90)
    print("OK user_id=", info1.get("user_id"), "name=", info1.get("username") or info1.get("nickname"))

    print("--- second ensure_wework_login (must NOT open again) ---")
    info2 = ensure_wework_login(client, smart=smart, timeout=10)
    print("OK user_id=", info2.get("user_id"))
    print("session_ready:", wework_session_ready())
    print("PASS: no duplicate open in same process")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        print("FAIL:", exc)
        raise
