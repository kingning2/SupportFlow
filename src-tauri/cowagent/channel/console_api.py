# encoding:utf-8
"""Console channel APIs (ported from channel/web/web_channel.py for Tauri stdio sidecar)."""

from __future__ import annotations

import json
import os
import threading
import time
from typing import Any

from channel import channel_manager
from common.log import logger
from config import conf


def _get_running_channel(name: str):
    try:
        mgr = channel_manager.get_channel_manager()
        if mgr:
            return mgr.get_channel(name)
    except Exception:
        pass
    return None


def _qr_to_data_uri(data: str) -> str:
    if not data:
        return ""
    try:
        import base64
        import io

        import qrcode as qr_lib

        qr = qr_lib.QRCode(error_correction=qr_lib.constants.ERROR_CORRECT_L, box_size=6, border=2)
        qr.add_data(data)
        qr.make(fit=True)
        img = qr.make_image(fill_color="black", back_color="white")
        buf = io.BytesIO()
        img.save(buf, format="PNG")
        b64 = base64.b64encode(buf.getvalue()).decode("ascii")
        return f"data:image/png;base64,{b64}"
    except ImportError:
        return ""


class _WeixinQrState:
    _qr_state: dict = {}

    @classmethod
    def get(cls) -> dict:
        return cls._qr_state

    @classmethod
    def set(cls, state: dict) -> None:
        cls._qr_state = state

    @classmethod
    def clear(cls) -> None:
        cls._qr_state = {}


def weixin_qrlogin_get() -> dict:
    try:
        running_ch = _get_running_channel("weixin")
        if running_ch and hasattr(running_ch, "_current_qr_url") and running_ch._current_qr_url:
            qr_image = _qr_to_data_uri(running_ch._current_qr_url)
            return {
                "status": "success",
                "qrcode_url": running_ch._current_qr_url,
                "qr_image": qr_image,
                "source": "channel",
            }

        from channel.weixin.weixin_api import DEFAULT_BASE_URL, WeixinApi

        base_url = conf().get("weixin_base_url", DEFAULT_BASE_URL)
        api = WeixinApi(base_url=base_url)
        qr_resp = api.fetch_qr_code()
        qrcode = qr_resp.get("qrcode", "")
        qrcode_url = qr_resp.get("qrcode_img_content", "")
        if not qrcode:
            return {"status": "error", "message": "No QR code returned"}
        qr_image = _qr_to_data_uri(qrcode_url)
        _WeixinQrState.set(
            {
                "qrcode": qrcode,
                "qrcode_url": qrcode_url,
                "base_url": base_url,
            }
        )
        return {"status": "success", "qrcode_url": qrcode_url, "qr_image": qr_image}
    except Exception as e:
        logger.error("[ConsoleApi] WeixinQr GET error: %s", e)
        return {"status": "error", "message": str(e)}


def weixin_qrlogin_post(body: dict) -> dict:
    action = body.get("action", "poll")
    if action == "refresh":
        return weixin_qrlogin_get()
    if action != "poll":
        return {"status": "error", "message": f"unknown action: {action}"}

    state = _WeixinQrState.get()
    qrcode = state.get("qrcode", "")
    base_url = state.get("base_url", "")
    if not qrcode:
        return {"status": "error", "message": "No active QR session"}

    from channel.weixin.weixin_api import DEFAULT_BASE_URL, WeixinApi

    api = WeixinApi(base_url=base_url or DEFAULT_BASE_URL)
    try:
        status_resp = api.poll_qr_status(qrcode, timeout=10)
    except Exception as e:
        return {"status": "error", "message": str(e)}

    qr_status = status_resp.get("status", "wait")

    if qr_status == "confirmed":
        bot_token = status_resp.get("bot_token", "")
        bot_id = status_resp.get("ilink_bot_id", "")
        result_base_url = status_resp.get("baseurl", base_url)
        user_id = status_resp.get("ilink_user_id", "")

        if not bot_token or not bot_id:
            return {"status": "error", "message": "Login confirmed but missing token"}

        cred_path = os.path.expanduser(
            conf().get("weixin_credentials_path", "~/.weixin_cow_credentials.json")
        )
        from channel.weixin.weixin_channel import _save_credentials

        _save_credentials(
            cred_path,
            {
                "token": bot_token,
                "base_url": result_base_url,
                "bot_id": bot_id,
                "user_id": user_id,
            },
        )
        conf()["weixin_token"] = bot_token
        conf()["weixin_base_url"] = result_base_url
        _WeixinQrState.clear()
        logger.info("[ConsoleApi] WeChat QR login confirmed: bot_id=%s", bot_id)
        return {"status": "success", "qr_status": "confirmed", "bot_id": bot_id}

    if qr_status == "expired":
        new_resp = api.fetch_qr_code()
        new_qrcode = new_resp.get("qrcode", "")
        new_qrcode_url = new_resp.get("qrcode_img_content", "")
        new_qr_image = _qr_to_data_uri(new_qrcode_url)
        state["qrcode"] = new_qrcode
        state["qrcode_url"] = new_qrcode_url
        _WeixinQrState.set(state)
        return {
            "status": "success",
            "qr_status": "expired",
            "qrcode_url": new_qrcode_url,
            "qr_image": new_qr_image,
        }

    return {"status": "success", "qr_status": qr_status}


def wx_qrlogin_get() -> dict:
    try:
        running_ch = _get_running_channel("wx")
        if running_ch and getattr(running_ch, "_current_qr_url", ""):
            qr_image = _qr_to_data_uri(running_ch._current_qr_url)
            return {
                "status": "success",
                "qrcode_url": running_ch._current_qr_url,
                "qr_image": qr_image,
                "login_status": getattr(running_ch, "login_status", "waiting_scan"),
                "source": "channel",
            }
        if running_ch and getattr(running_ch, "login_status", "") == "logged_in":
            return {
                "status": "success",
                "qr_status": "confirmed",
                "login_status": "logged_in",
            }
        return {
            "status": "success",
            "login_status": getattr(running_ch, "login_status", "idle") if running_ch else "idle",
            "message": "Start wx channel first or wait for QR",
        }
    except Exception as e:
        logger.error("[ConsoleApi] WxQr GET error: %s", e)
        return {"status": "error", "message": str(e)}


def wx_qrlogin_post(body: dict) -> dict:
    try:
        running_ch = _get_running_channel("wx")
        if not running_ch:
            return {"status": "error", "message": "wx channel not running"}

        login_status = getattr(running_ch, "login_status", "idle")
        if login_status == "logged_in":
            return {"status": "success", "qr_status": "confirmed", "login_status": login_status}
        if login_status == "scanned":
            return {"status": "success", "qr_status": "scaned", "login_status": login_status}

        qr_url = getattr(running_ch, "_current_qr_url", "")
        qr_image = _qr_to_data_uri(qr_url) if qr_url else ""
        return {
            "status": "success",
            "qr_status": "wait",
            "login_status": login_status,
            "qrcode_url": qr_url,
            "qr_image": qr_image,
        }
    except Exception as e:
        logger.error("[ConsoleApi] WxQr POST error: %s", e)
        return {"status": "error", "message": str(e)}


class _FeishuRegisterState:
    _state: dict = {}
    _lock = threading.Lock()

    @classmethod
    def start_register_thread(cls) -> None:
        with cls._lock:
            old_cancel = cls._state.get("cancel_event") if cls._state else None
            if old_cancel is not None:
                old_cancel.set()
            cancel_event = threading.Event()
            cls._state = {"status": "starting", "cancel_event": cancel_event}

        def _worker():
            try:
                import lark_oapi as lark
            except ImportError:
                with cls._lock:
                    cls._state["status"] = "error"
                    cls._state["error"] = "lark-oapi SDK 未安装，请执行 pip install -U lark-oapi"
                return

            def _on_qr(info):
                with cls._lock:
                    cls._state["url"] = info.get("url", "")
                    cls._state["expire_in"] = info.get("expire_in", 600)
                    cls._state["qr_image"] = _qr_to_data_uri(info.get("url", ""))
                    cls._state["status"] = "pending"
                logger.info("[FeishuRegister] QR ready, expire_in=%ss", info.get("expire_in"))

            def _on_status(info):
                status = info.get("status")
                if status == "polling":
                    return
                logger.info("[FeishuRegister] SDK status: %s", info)

            try:
                from common.http_proxy import bypass_system_proxy

                with bypass_system_proxy():
                    result = lark.register_app(
                        on_qr_code=_on_qr,
                        on_status_change=_on_status,
                        source="SupportFlow",
                        cancel_event=cancel_event,
                    )
                with cls._lock:
                    cls._state["status"] = "done"
                    cls._state["app_id"] = result.get("client_id", "")
                    cls._state["app_secret"] = result.get("client_secret", "")
                logger.info("[FeishuRegister] App created: app_id=%s", result.get("client_id"))
            except Exception as e:
                err_msg = str(e)
                err_cls = e.__class__.__name__
                if "Expired" in err_cls:
                    status = "expired"
                elif "Denied" in err_cls:
                    status = "denied"
                elif "abort" in err_msg.lower() or "cancel" in err_msg.lower():
                    return
                else:
                    status = "error"
                with cls._lock:
                    if cls._state.get("cancel_event") is cancel_event:
                        cls._state["status"] = status
                        cls._state["error"] = err_msg
                logger.warning("[FeishuRegister] Register failed (%s): %s", err_cls, err_msg)

        threading.Thread(target=_worker, daemon=True, name="feishu-register").start()


def feishu_register_get() -> dict:
    try:
        _FeishuRegisterState.start_register_thread()
        for _ in range(100):
            with _FeishuRegisterState._lock:
                if _FeishuRegisterState._state.get("url") or _FeishuRegisterState._state.get("status") in (
                    "error",
                    "expired",
                    "denied",
                ):
                    break
            time.sleep(0.1)
        with _FeishuRegisterState._lock:
            if _FeishuRegisterState._state.get("status") in ("error", "expired", "denied"):
                return {
                    "status": "error",
                    "message": _FeishuRegisterState._state.get("error", "register failed"),
                }
            if not _FeishuRegisterState._state.get("url"):
                return {"status": "error", "message": "等待飞书二维码超时，请重试"}
            return {
                "status": "success",
                "qrcode_url": _FeishuRegisterState._state["url"],
                "qr_image": _FeishuRegisterState._state.get("qr_image", ""),
                "expire_in": _FeishuRegisterState._state.get("expire_in", 600),
            }
    except Exception as e:
        logger.error("[ConsoleApi] FeishuRegister GET error: %s", e)
        return {"status": "error", "message": str(e)}


def feishu_register_post(body: dict) -> dict:
    action = body.get("action", "poll")
    if action != "poll":
        return {"status": "error", "message": f"unknown action: {action}"}
    try:
        with _FeishuRegisterState._lock:
            status = _FeishuRegisterState._state.get("status", "idle")
            if status == "done":
                payload = {
                    "status": "success",
                    "register_status": "done",
                    "app_id": _FeishuRegisterState._state.get("app_id", ""),
                    "app_secret": _FeishuRegisterState._state.get("app_secret", ""),
                }
                _FeishuRegisterState._state = {}
                return payload
            if status in ("error", "expired", "denied"):
                return {
                    "status": "success",
                    "register_status": status,
                    "message": _FeishuRegisterState._state.get("error", ""),
                }
            return {"status": "success", "register_status": "pending"}
    except Exception as e:
        logger.error("[ConsoleApi] FeishuRegister POST error: %s", e)
        return {"status": "error", "message": str(e)}


def dispatch_console_api(path: str, method: str, body: dict | None = None) -> dict:
    """Route console API paths used by the channels web UI."""
    path = (path or "").strip().lstrip("/")
    method = (method or "GET").upper()
    body = body or {}

    routes = {
        ("weixin/qrlogin", "GET"): weixin_qrlogin_get,
        ("weixin/qrlogin", "POST"): lambda: weixin_qrlogin_post(body),
        ("wx/qrlogin", "GET"): wx_qrlogin_get,
        ("wx/qrlogin", "POST"): lambda: wx_qrlogin_post(body),
        ("feishu/register", "GET"): feishu_register_get,
        ("feishu/register", "POST"): lambda: feishu_register_post(body),
    }
    handler = routes.get((path, method))
    if not handler:
        return {"status": "error", "message": f"unknown console api: {method} /{path}"}
    result = handler()
    if not isinstance(result, dict):
        return {"status": "error", "message": "invalid handler result"}
    return result
