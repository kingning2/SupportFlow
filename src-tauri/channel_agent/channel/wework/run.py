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


def reset_wework_client() -> None:
    """Drop client so the next get_wework() reapplies config (channel restart)."""
    global wework, _handlers_registered_for
    wework = None
    _handlers_registered_for = None
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


def wework_session_ready() -> bool:
    """True when the global ntwork client has valid login info."""
    client = wework
    if client is None:
        return False
    try:
        info = client.get_login_info()
        return bool(info and info.get("user_id"))
    except Exception:
        return False


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
