# encoding:utf-8
"""Personal WeChat channel via vendored itchat (channel_type=wx)."""

import io
import json
import os
import threading
import time
import base64

import requests
import qrcode as qr_lib

from bridge.context import ContextType
from bridge.reply import Reply, ReplyType
from channel.chat_channel import ChatChannel
from channel import chat_channel
from channel.chat_message import ChatMessage
from channel.wechat.wechat_message import WechatMessage
from common.expired_dict import ExpiredDict
from common.log import logger
from common.singleton import singleton
from common.time_check import time_checker
from common.utils import convert_webp_to_png, remove_markdown_symbol
from config import conf, get_appdata_dir
from lib import itchat
from lib.itchat.content import *

def _wx_http_session():
    """HTTP session for wx media download; honors global use_proxy / proxy config."""
    from common.http_proxy import get_http_session

    return get_http_session()


@itchat.msg_register([TEXT, VOICE, PICTURE, NOTE, ATTACHMENT, SHARING])
def handler_single_msg(msg):
    try:
        cmsg = WechatMessage(msg, False)
    except NotImplementedError as e:
        logger.debug("[WX]single message {} skipped: {}".format(msg["MsgId"], e))
        return None
    WechatChannel().handle_single(cmsg)
    return None


@itchat.msg_register([TEXT, VOICE, PICTURE, NOTE, ATTACHMENT, SHARING], isGroupChat=True)
def handler_group_msg(msg):
    try:
        cmsg = WechatMessage(msg, True)
    except NotImplementedError as e:
        logger.debug("[WX]group message {} skipped: {}".format(msg["MsgId"], e))
        return None
    WechatChannel().handle_group(cmsg)
    return None


def _check(func):
    def wrapper(self, cmsg: ChatMessage):
        msg_id = cmsg.msg_id
        if msg_id in self.receivedMsgs:
            logger.info("Wechat message {} already received, ignore".format(msg_id))
            return
        self.receivedMsgs[msg_id] = True
        create_time = cmsg.create_time
        if conf().get("hot_reload") and int(create_time) < int(time.time()) - 60:
            logger.debug("[WX]history message {} skipped".format(msg_id))
            return
        if cmsg.my_msg and not cmsg.is_group:
            logger.debug("[WX]my message {} skipped".format(msg_id))
            return
        return func(self, cmsg)

    return wrapper


def _module_qr_callback(uuid, status, qrcode):
    """Update channel QR/login state for web console and terminal."""
    ch = WechatChannel()
    if status == "0" and uuid:
        url = f"https://login.weixin.qq.com/l/{uuid}"
        ch._current_qr_url = url
        ch.login_status = "waiting_scan"
        ch.notify_channel_status(
            "waiting_scan",
            qr_code_url=url,
            qr_image=_qr_to_data_uri(url),
        )
        logger.info(f"[WechatChannel] QR ready: {url}")
        _print_terminal_qr(uuid)
        _notify_cloud_qrcode(url)
    elif status == "201":
        ch.login_status = "scanned"
        ch.notify_channel_status(
            "scanned",
            qr_code_url=ch._current_qr_url,
            qr_image=_qr_to_data_uri(ch._current_qr_url),
        )
    elif status == "200":
        ch.login_status = "logged_in"
        ch._current_qr_url = ""
        ch.notify_channel_status("logged_in")


def _qr_to_data_uri(data: str) -> str:
    """Convert a QR URL into an inline PNG data URI for the frontend."""
    if not data:
        return ""
    qr = qr_lib.QRCode(error_correction=qr_lib.constants.ERROR_CORRECT_L, box_size=6, border=2)
    qr.add_data(data)
    qr.make(fit=True)
    img = qr.make_image(fill_color="black", back_color="white")
    buf = io.BytesIO()
    img.save(buf, format="PNG")
    b64 = base64.b64encode(buf.getvalue()).decode("ascii")
    return f"data:image/png;base64,{b64}"


def _print_terminal_qr(uuid):
    """Log QR link for terminal users and print an ASCII QR."""
    url = f"https://login.weixin.qq.com/l/{uuid}"
    print("\n" + "=" * 60)
    print("  个人微信(itchat) 请扫码登录")
    print(f"  Web 控制台: http://127.0.0.1:{port}  → 通道 → 查看二维码")
    print(f"  或直接打开链接: {url}")
    print("=" * 60)
    # Optional ASCII QR when not using web UI (no PIL Image.show — avoids popping OS photo viewer)
    try:
        import qrcode as qr_lib
        qr = qr_lib.QRCode(border=1)
        qr.add_data(url)
        qr.make(fit=True)
        qr.print_ascii(invert=True)
    except UnicodeEncodeError:
        pass


@singleton
class WechatChannel(ChatChannel):
    NOT_SUPPORT_REPLYTYPE = []

    LOGIN_STATUS_IDLE = "idle"
    LOGIN_STATUS_WAITING = "waiting_scan"
    LOGIN_STATUS_SCANNED = "scanned"
    LOGIN_STATUS_OK = "logged_in"

    def __init__(self):
        super().__init__()
        self.receivedMsgs = ExpiredDict(conf().get("expires_in_seconds", 3600))
        self.auto_login_times = 0
        self._stop_event = threading.Event()
        self.login_status = self.LOGIN_STATUS_IDLE
        self._current_qr_url = ""

    def startup(self):
        self._stop_event.clear()
        logger.info("[WechatChannel] Starting itchat (channel_type=wx)...")
        self.login_status = self.LOGIN_STATUS_WAITING
        self.notify_channel_status("waiting_scan")

        hot_reload = conf().get("hot_reload", False)
        status_path = os.path.join(get_appdata_dir(), "itchat.pkl")

        try:
            itchat.instance.receivingRetryCount = 600
            from common.http_proxy import get_http_session

            wx_http = get_http_session()
            itchat.instance.s.trust_env = wx_http.trust_env
            itchat.instance.s.proxies = dict(wx_http.proxies)
            itchat.auto_login(
                enableCmdQR=2,
                hotReload=hot_reload,
                statusStorageDir=status_path,
                qrCallback=_module_qr_callback,
                exitCallback=self.exitCallback,
                loginCallback=self.loginCallback,
            )
            if self._stop_event.is_set():
                return
            self.user_id = itchat.instance.storageClass.userName
            self.name = itchat.instance.storageClass.nickName
            self.login_status = self.LOGIN_STATUS_OK
            self._current_qr_url = ""
            self.notify_channel_status(
                "logged_in",
                user_id=str(self.user_id),
                display_name=str(self.name),
            )
            logger.info(
                "[WechatChannel] Login success, user_id=%s nickname=%s",
                self.user_id,
                self.name,
            )
            self.report_startup_success()
            _notify_cloud_connected()
            itchat.run()
        except Exception as e:
            self.login_status = self.LOGIN_STATUS_IDLE
            self.notify_channel_status("stopped", message=str(e))
            if not self._stop_event.is_set():
                logger.exception(f"[WechatChannel] startup failed: {e}")

    def stop(self):
        logger.info("[WechatChannel] stop() called")
        self._stop_event.set()
        self.login_status = self.LOGIN_STATUS_IDLE
        self._current_qr_url = ""
        self.notify_channel_status("stopped")
        try:
            itchat.logout()
        except Exception as e:
            logger.debug(f"[WechatChannel] logout: {e}")

    def exitCallback(self):
        self.auto_login_times += 1
        if self.auto_login_times < 100 and not self._stop_event.is_set():
            chat_channel.handler_pool._shutdown = False
            self.startup()

    def loginCallback(self):
        logger.info("[WechatChannel] loginCallback")
        self.login_status = self.LOGIN_STATUS_OK
        _send_login_success()

    @time_checker
    @_check
    def handle_single(self, cmsg: ChatMessage):
        if cmsg.other_user_id in ["weixin"]:
            return
        if cmsg.ctype == ContextType.VOICE:
            if conf().get("speech_recognition") is not True:
                return
            logger.debug("[WX]receive voice msg: {}".format(cmsg.content))
        elif cmsg.ctype == ContextType.IMAGE:
            logger.debug("[WX]receive image msg: {}".format(cmsg.content))
        elif cmsg.ctype == ContextType.PATPAT:
            logger.debug("[WX]receive patpat msg: {}".format(cmsg.content))
        elif cmsg.ctype == ContextType.TEXT:
            logger.debug(
                "[WX]receive text msg: {}, cmsg={}".format(
                    json.dumps(cmsg._rawmsg, ensure_ascii=False), cmsg
                )
            )
        else:
            logger.debug("[WX]receive msg: {}, cmsg={}".format(cmsg.content, cmsg))
        context = self._compose_context(cmsg.ctype, cmsg.content, isgroup=False, msg=cmsg)
        if context:
            self.produce(context)

    @time_checker
    @_check
    def handle_group(self, cmsg: ChatMessage):
        if cmsg.ctype == ContextType.VOICE:
            if conf().get("group_speech_recognition") is not True:
                return
            logger.debug("[WX]receive voice for group msg: {}".format(cmsg.content))
        elif cmsg.ctype == ContextType.IMAGE:
            logger.debug("[WX]receive image for group msg: {}".format(cmsg.content))
        elif cmsg.ctype in [
            ContextType.JOIN_GROUP,
            ContextType.PATPAT,
            ContextType.ACCEPT_FRIEND,
            ContextType.EXIT_GROUP,
        ]:
            logger.debug("[WX]receive note msg: {}".format(cmsg.content))
        elif cmsg.ctype == ContextType.TEXT:
            pass
        elif cmsg.ctype == ContextType.FILE:
            logger.debug(f"[WX]receive attachment msg, file_name={cmsg.content}")
        else:
            logger.debug("[WX]receive group msg: {}".format(cmsg.content))
        context = self._compose_context(
            cmsg.ctype,
            cmsg.content,
            isgroup=True,
            msg=cmsg,
            no_need_at=conf().get("no_need_at", False),
        )
        if context:
            self.produce(context)

    def send(self, reply: Reply, context):
        receiver = context["receiver"]
        if reply.type == ReplyType.TEXT:
            reply.content = remove_markdown_symbol(reply.content)
            itchat.send(reply.content, toUserName=receiver)
            logger.info("[WX] sendMsg={}, receiver={}".format(reply, receiver))
        elif reply.type in (ReplyType.ERROR, ReplyType.INFO):
            reply.content = remove_markdown_symbol(reply.content)
            itchat.send(reply.content, toUserName=receiver)
            logger.info("[WX] sendMsg={}, receiver={}".format(reply, receiver))
        elif reply.type == ReplyType.VOICE:
            itchat.send_file(reply.content, toUserName=receiver)
            logger.info("[WX] sendFile={}, receiver={}".format(reply.content, receiver))
        elif reply.type == ReplyType.IMAGE_URL:
            img_url = reply.content
            logger.debug(f"[WX] start download image, img_url={img_url}")
            pic_res = _wx_http_session().get(img_url, stream=True, timeout=60)
            image_storage = io.BytesIO()
            for block in pic_res.iter_content(1024):
                image_storage.write(block)
            image_storage.seek(0)
            if ".webp" in img_url:
                try:
                    image_storage = convert_webp_to_png(image_storage)
                except Exception as e:
                    logger.error(f"Failed to convert image: {e}")
                    return
            itchat.send_image(image_storage, toUserName=receiver)
            logger.info("[WX] sendImage url={}, receiver={}".format(img_url, receiver))
        elif reply.type == ReplyType.IMAGE:
            image_storage = reply.content
            image_storage.seek(0)
            itchat.send_image(image_storage, toUserName=receiver)
            logger.info("[WX] sendImage, receiver={}".format(receiver))
        elif reply.type == ReplyType.FILE:
            itchat.send_file(reply.content, toUserName=receiver)
            logger.info("[WX] sendFile, receiver={}".format(receiver))
        elif reply.type == ReplyType.VIDEO:
            itchat.send_video(reply.content, toUserName=receiver)
            logger.info("[WX] sendVideo, receiver={}".format(receiver))
        elif reply.type == ReplyType.VIDEO_URL:
            video_url = reply.content
            video_res = _wx_http_session().get(video_url, stream=True, timeout=120)
            video_storage = io.BytesIO()
            for block in video_res.iter_content(1024):
                video_storage.write(block)
            video_storage.seek(0)
            itchat.send_video(video_storage, toUserName=receiver)
            logger.info("[WX] sendVideo url={}, receiver={}".format(video_url, receiver))


def _send_login_success():
    return None


def _send_logout():
    return None


def _notify_cloud_qrcode(url: str):
    return None


def _notify_cloud_connected():
    return None
