"""
Global ntwork WeWork client singleton.

Imported by wework_channel; optional dependency (Windows + desktop WeCom).
Client is created lazily so config (e.g. wework_exe_path) is applied first.
"""
import os
import sys
import time

from common.log import logger

os.environ.setdefault("ntwork_LOG", "ERROR")

try:
    import ntwork
except ImportError:
    ntwork = None

# Lazy singleton; do not instantiate at import time.
wework = None
# True after open()/attach() in this process — never call open twice.
_hook_started = False

DEFAULT_WEWORK_VERSION = "4.0.8.6027"
_WEWORK_EXE_NAMES = ("WXWork.exe", "wework.exe", "WeCom.exe")
_handlers_registered_for = None


def _resolve_wework_exe_path(raw: str) -> str:
    """Expand path; accept install dir or path to WXWork.exe."""
    from common.utils import expand_path

    path = expand_path((raw or "").strip())
    if not path:
        return ""
    if os.path.isfile(path):
        return path
    if os.path.isdir(path):
        for name in _WEWORK_EXE_NAMES:
            candidate = os.path.join(path, name)
            if os.path.isfile(candidate):
                return candidate
        raise FileNotFoundError(
            f"No WeCom executable ({', '.join(_WEWORK_EXE_NAMES)}) under: {path}"
        )
    raise FileNotFoundError(f"WeCom path not found: {path}")


def _discover_ntwork_wework_exe(target_version: str = DEFAULT_WEWORK_VERSION) -> str:
    """
    When multiple WeCom installs exist, registry "Executable" may point at a newer build.
    Prefer the versioned subfolder that matches ntwork (e.g. .../4.0.8.6027/WXWork.exe).
    """
    if sys.platform != "win32":
        return ""

    try:
        import winreg
    except ImportError:
        return ""

    exe_reg = ""
    wemeet_dir = ""
    try:
        with winreg.OpenKey(winreg.HKEY_CURRENT_USER, r"Software\Tencent\WXWork") as key:
            try:
                exe_reg, _ = winreg.QueryValueEx(key, "Executable")
            except OSError:
                pass
            try:
                wemeet_dir, _ = winreg.QueryValueEx(key, "WeMeetDir")
            except OSError:
                pass
    except OSError:
        return ""

    candidates = []
    if exe_reg:
        install_root = os.path.dirname(exe_reg)
        candidates.append(os.path.join(install_root, target_version, "WXWork.exe"))
    if wemeet_dir and target_version in wemeet_dir.replace("/", os.sep):
        version_root = os.path.dirname(os.path.dirname(wemeet_dir))
        candidates.append(os.path.join(version_root, "WXWork.exe"))

    for path in candidates:
        if path and os.path.isfile(path):
            logger.info(f"[Wework] Auto-detected ntwork WeCom: {path}")
            return path
    return ""


def _format_wework_init_error(exc: BaseException) -> str:
    """Turn ntwork exceptions (often empty str) into actionable messages."""
    from config import conf

    name = type(exc).__name__
    detail = str(exc).strip()
    detected = ""
    if ntwork is not None:
        try:
            detected = ntwork.get_install_wework_version() or ""
        except Exception:
            pass

    if name == "WeWorkVersionNotMatchError":
        lines = [
            "企业微信版本与 ntwork 不匹配。",
            f"ntwork 需要版本 {DEFAULT_WEWORK_VERSION}，",
        ]
        if detected:
            lines.append(f"当前注册表默认客户端版本为 {detected}。")
        lines.append(
            "本机若安装多个企微，请在配置中设置 wework_exe_path，"
            f"指向 {DEFAULT_WEWORK_VERSION} 的 WXWork.exe（例如 D:\\WXWork\\{DEFAULT_WEWORK_VERSION}\\WXWork.exe）。"
        )
        return "".join(lines)

    if isinstance(exc, FileNotFoundError):
        return f"{detail or name}。请检查 wework_exe_path 是否正确。"

    if detail:
        return f"{name}: {detail}"
    configured = (conf().get("wework_exe_path") or "").strip()
    if configured:
        return f"{name}（已配置 wework_exe_path={configured}）"
    return f"{name}。请设置 wework_exe_path 指向 {DEFAULT_WEWORK_VERSION} 的 WXWork.exe。"


def _apply_ntwork_path_from_config() -> None:
    """Call ntwork.set_wework_exe_path before the first WeWork / WeWorkMgr init."""
    if ntwork is None:
        return
    from config import conf

    raw = (conf().get("wework_exe_path") or "").strip()
    version = (conf().get("wework_version") or "").strip() or None
    exe_path = None

    if raw:
        exe_path = _resolve_wework_exe_path(raw)
    else:
        discovered = _discover_ntwork_wework_exe(
            version or DEFAULT_WEWORK_VERSION
        )
        if discovered:
            exe_path = discovered

    if exe_path:
        if version is None:
            version = DEFAULT_WEWORK_VERSION
        logger.info(f"[Wework] Using WeCom executable: {exe_path} (version {version})")
        ntwork.set_wework_exe_path(exe_path, version)
    elif version:
        ntwork.set_wework_exe_path(None, version)


def _find_running_wework_pids() -> list:
    """
    Return WXWork.exe PIDs with desktop UI processes first.

    tasklist order often hits a background process (no login session) before the
    main window; prefer Get-Process entries that have MainWindowTitle.
    """
    if sys.platform != "win32":
        return []

    pids: list = []
    try:
        import subprocess

        out = subprocess.check_output(
            [
                "powershell",
                "-NoProfile",
                "-Command",
                "Get-Process WXWork -ErrorAction SilentlyContinue | "
                "Where-Object { $_.MainWindowTitle } | "
                "Sort-Object WorkingSet64 -Descending | "
                "ForEach-Object { $_.Id }",
            ],
            text=True,
            errors="replace",
            timeout=15,
        )
        for line in out.strip().splitlines():
            token = line.strip()
            if token.isdigit():
                pids.append(int(token))
    except Exception as exc:
        logger.debug("[Wework] list UI WXWork PIDs: %s", exc)

    if pids:
        return pids

    try:
        import subprocess

        out = subprocess.check_output(
            ["tasklist", "/FI", "IMAGENAME eq WXWork.exe", "/FO", "CSV", "/NH"],
            text=True,
            errors="replace",
            timeout=10,
        )
        for line in out.strip().splitlines():
            parts = [p.strip('"') for p in line.split('","')]
            if len(parts) >= 2 and parts[0].lower() == "wxwork.exe":
                pid = int(parts[1])
                if pid not in pids:
                    pids.append(pid)
    except Exception as exc:
        logger.debug("[Wework] find WXWork.exe pid: %s", exc)
    return pids


def _find_running_wework_pid() -> int:
    """Return the best PID to hook, or 0."""
    found = _find_running_wework_pids()
    return found[0] if found else 0


def _wait_client_bind(client, timeout: float) -> bool:
    deadline = time.time() + timeout
    while time.time() < deadline:
        if getattr(client, "client_id", 0):
            return True
        time.sleep(0.2)
    return False


def _apply_login_info(client, info: dict) -> dict:
    """Mirror ntwork MT_USER_LOGIN_MSG side effects for already-logged-in desktop."""
    if not isinstance(info, dict) or not info.get("user_id"):
        return info
    client.login_status = True
    try:
        client._WeWork__login_info = info  # noqa: SLF001
        client._WeWork__wait_login_event.set()
    except Exception:
        pass
    return info


def reset_wework_client() -> None:
    """Drop client so the next get_wework() reapplies config (channel restart)."""
    global wework, _handlers_registered_for, _hook_started
    wework = None
    _handlers_registered_for = None
    _hook_started = False
    if ntwork is None:
        return
    try:
        from ntwork.core.mgr import WeWorkMgr
        from ntwork.utils.singleton import Singleton

        Singleton._instances.pop(WeWorkMgr, None)
    except Exception:
        pass


def get_wework():
    """Return ntwork WeWork client, creating it on first use."""
    global wework
    if ntwork is None:
        return None
    if wework is not None:
        return wework
    _apply_ntwork_path_from_config()
    wework = ntwork.WeWork()
    return wework


def wework_login_info(client=None):
    """Return login_info dict when ntwork already received MT_USER_LOGIN_MSG."""
    client = client if client is not None else wework
    if client is None:
        return None
    try:
        info = client.get_login_info()
        if isinstance(info, dict) and info.get("user_id"):
            return info
    except Exception:
        pass
    return None


def _fetch_login_via_api(client, timeout: float = 30.0):
    """
    After hook bind, ask the desktop client for self profile.
    Works when the user logged in before we attached (no MT_USER_LOGIN_MSG).

    ntwork gates get_self_info on login_status; set it after client_id bind.
    """
    try:
        from ntwork.exception import WeWorkNotLoginError
    except ImportError:
        WeWorkNotLoginError = Exception  # type: ignore

    deadline = time.time() + timeout
    while time.time() < deadline:
        if not getattr(client, "client_id", 0):
            time.sleep(0.3)
            continue
        if not client.login_status:
            client.login_status = True
        try:
            info = client.get_self_info()
            if isinstance(info, dict) and info.get("user_id"):
                return _apply_login_info(client, info)
        except WeWorkNotLoginError:
            client.login_status = False
        except Exception as exc:
            logger.debug("[Wework] get_self_info: %s", exc)
        time.sleep(0.3)
    return None


def wework_session_ready() -> bool:
    """True when the global ntwork client is hooked and logged in."""
    client = wework
    if client is None:
        return False
    if wework_login_info(client):
        return True
    return _fetch_login_via_api(client, timeout=1.0) is not None


def _hook_wework_client(client, smart: bool, timeout: float) -> None:
    """
    Hook the desktop client. Try one attach to the main-window WXWork.exe, then
    open(smart) (same as upstream CowAgent). Avoid attach/detach loops — wcprobe
    can crash when hopping between PIDs.
    """
    if smart:
        pids = _find_running_wework_pids()
        if pids:
            pid = pids[0]
            if client.attach(pid):
                logger.info(
                    "[Wework] Attached to WXWork.exe pid=%s (desktop UI)", pid
                )
                bind_budget = min(20.0, timeout * 0.25)
                if _wait_client_bind(client, bind_budget):
                    if _fetch_login_via_api(
                        client, timeout=min(15.0, timeout * 0.2)
                    ):
                        return
                logger.warning(
                    "[Wework] attach(pid=%s) bound but no login session yet; "
                    "falling back to open(smart=True)",
                    pid,
                )
            else:
                logger.warning("[Wework] attach(pid=%s) returned false", pid)

    if smart:
        logger.info("[Wework] open(smart=True)")
    else:
        logger.info("[Wework] open(smart=False) — new WeCom instance")
    if not client.open(smart):
        raise RuntimeError("WeCom open() returned pid=0")
    logger.info("[Wework] open(smart=%s) pid=%s", smart, client.pid)


def ensure_wework_login(client, smart: bool = True, timeout: float = 120.0) -> dict:
    """
    Hook WeCom once per process. Prefer attach() to the desktop UI WXWork.exe;
    fall back to open(smart) like upstream CowAgent when attach cannot see a session.
    """
    global _hook_started

    started_at = time.time()

    info = wework_login_info(client)
    if info:
        logger.info(
            "[Wework] Reuse in-process login user_id=%s",
            info.get("user_id"),
        )
        return info

    api_info = _fetch_login_via_api(client, timeout=1.0)
    if api_info:
        logger.info(
            "[Wework] Reuse hooked session via get_self_info user_id=%s",
            api_info.get("user_id"),
        )
        return api_info

    if not _hook_started:
        _hook_wework_client(client, smart, timeout)
        _hook_started = True
    else:
        logger.info("[Wework] Hook already started in this process (pid=%s)", client.pid)

    bind_left = max(5.0, min(30.0, timeout - (time.time() - started_at)))
    if not _wait_client_bind(client, bind_left):
        raise RuntimeError(
            "WeCom hook bind timeout (client_id is still 0). "
            "Quit all WXWork.exe processes, start a single WeCom 4.0.8.6027 client, "
            "log in, then restart the channel."
        )

    api_info = _fetch_login_via_api(
        client, timeout=max(10.0, timeout - (time.time() - started_at) - 5.0)
    )
    if api_info:
        logger.info(
            "[Wework] Logged in via desktop session user_id=%s",
            api_info.get("user_id"),
        )
        return api_info

    info = wework_login_info(client)
    if info:
        return info

    remain = max(5.0, timeout - (time.time() - started_at))
    logger.info("[Wework] Waiting for WeCom login event (%.0fs max)...", remain)
    client.wait_login(timeout=remain)

    info = wework_login_info(client) or {}
    if info.get("user_id"):
        return info

    api_info = _fetch_login_via_api(client, timeout=10.0)
    if api_info:
        return api_info

    raise RuntimeError(
        "WeCom hook connected but login_info is empty. "
        "If you are already logged in, quit every WXWork.exe (Task Manager), open one "
        "WeCom 4.0.8.6027 window, log in again, then restart the channel. "
        "Multiple WXWork processes or hooking a background pid causes this."
    )


def init_wework_client():
    """Create client; raises with a readable message on failure."""
    try:
        client = get_wework()
    except Exception as exc:
        raise RuntimeError(_format_wework_init_error(exc)) from exc
    if client is None:
        raise RuntimeError("ntwork is not available")
    return client


def register_message_handlers(client) -> None:
    """Register recv callbacks on the given WeWork instance (once per instance)."""
    global _handlers_registered_for
    if client is None or ntwork is None:
        return
    if _handlers_registered_for is client:
        return

    from channel.wework.wework_channel import on_wework_message

    @client.msg_register([
        ntwork.MT_RECV_TEXT_MSG,
        ntwork.MT_RECV_IMAGE_MSG,
        11072,
        ntwork.MT_RECV_LINK_CARD_MSG,
        ntwork.MT_RECV_FILE_MSG,
        ntwork.MT_RECV_VOICE_MSG,
    ])
    def all_msg_handler(wework_instance, message):
        on_wework_message(wework_instance, message)

    _handlers_registered_for = client


def run_until_stopped(stop_event) -> None:
    """Keep process alive while ntwork callbacks run; exit when stop_event is set."""
    if not ntwork:
        return
    try:
        while not stop_event.is_set():
            time.sleep(0.1)
    except KeyboardInterrupt:
        pass
