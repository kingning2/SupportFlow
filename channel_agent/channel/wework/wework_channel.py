# encoding:utf-8
"""
WeCom desktop client channel (ntwork).

Hooks the logged-in Windows WeCom PC client. Supports direct chat and group chat.
Requires: pip install ntwork pilk (Windows).
"""

import io
import json
import os
import random
import re
import tempfile
import threading
import time
import uuid

import requests

from bridge.context import Context, ContextType
from bridge.reply import Reply, ReplyType
from channel.chat_channel import ChatChannel
from channel.chat_message import ChatMessage
from channel.rust_ipc import call_rust, call_rust_bool
from channel.wework.run import (
    get_wework,
    init_wework_client,
    register_message_handlers,
    run_until_stopped,
)
from channel.wework.wework_message import WeworkMessage, get_with_retry
from common.expired_dict import ExpiredDict
from common.log import logger
from common.singleton import singleton
from common.time_check import time_checker
from common.utils import compress_imgfile, expand_path, fsize
from config import conf

try:
    import ntwork
except ImportError:
    ntwork = None


def _tmp_dir() -> str:
    ws_root = expand_path(conf().get("agent_workspace", "~/supportflow"))
    tmp_dir = os.path.join(ws_root, "tmp")
    os.makedirs(tmp_dir, exist_ok=True)
    return tmp_dir


def _interruptible_sleep(seconds: float, stop_event: threading.Event) -> bool:
    """Sleep in 0.1s slices. Returns False if stop_event was set."""
    end = time.time() + seconds
    while time.time() < end:
        if stop_event.is_set():
            return False
        time.sleep(min(0.1, end - time.time()))
    return True


def _write_sync_snapshot(client, directory: str) -> None:
    """Fetch and persist WeCom contacts / rooms / members into local snapshot files."""
    contacts = get_with_retry(client.get_external_contacts)
    rooms = get_with_retry(client.get_rooms)
    if not contacts or not rooms:
        raise RuntimeError("Failed to fetch contacts or rooms from WeCom client")

    with open(os.path.join(directory, "wework_contacts.json"), "w", encoding="utf-8") as f:
        json.dump(contacts, f, ensure_ascii=False, indent=4)
    with open(os.path.join(directory, "wework_rooms.json"), "w", encoding="utf-8") as f:
        json.dump(rooms, f, ensure_ascii=False, indent=4)

    result = {}
    for room in rooms.get("room_list", []):
        room_wxid = room["conversation_id"]
        result[room_wxid] = client.get_room_members(room_wxid)
    with open(os.path.join(directory, "wework_room_members.json"), "w", encoding="utf-8") as f:
        json.dump(result, f, ensure_ascii=False, indent=4)


def _contacts_sync_done(user_id: str) -> bool:
    """Return whether contact sync has already completed for this WeCom account."""
    if not user_id:
        return False
    try:
        return call_rust_bool("wework.contacts_synced", {"wework_user_id": user_id}, timeout=15.0)
    except Exception as e:
        logger.warning("[Wework] query contacts synced state failed: %s", e)
        return False


def _mark_contacts_sync_done(user_id: str) -> None:
    """Persist successful contact sync marker for this WeCom account."""
    if not user_id:
        return
    try:
        call_rust(
            "wework.mark_contacts_synced",
            {"wework_user_id": user_id, "synced_at": int(time.time() * 1000)},
            timeout=15.0,
        )
    except Exception as e:
        logger.warning("[Wework] mark contacts synced failed: %s", e)


def download_and_compress_image(url, filename):
    directory = _tmp_dir()
    pic_res = requests.get(url, stream=True, timeout=60)
    image_storage = io.BytesIO()
    for block in pic_res.iter_content(1024):
        image_storage.write(block)
    sz = fsize(image_storage)
    if sz >= 10 * 1024 * 1024:
        logger.info(f"[Wework] image too large, compressing, sz={sz}")
        image_storage = compress_imgfile(image_storage, 10 * 1024 * 1024 - 1)
    image_storage.seek(0)
    from PIL import Image
    image = Image.open(image_storage)
    image_path = os.path.join(directory, f"{filename}.png")
    image.save(image_path, "png")
    return image_path


def download_video(url, filename):
    directory = _tmp_dir()
    response = requests.get(url, stream=True, timeout=120)
    video_path = os.path.join(directory, f"{filename}.mp4")
    total_size = 0
    with open(video_path, "wb") as f:
        for block in response.iter_content(1024):
            total_size += len(block)
            if total_size > 30 * 1024 * 1024:
                logger.info("[Wework] video larger than 30MB, skip")
                try:
                    os.remove(video_path)
                except OSError:
                    pass
                return None
            f.write(block)
    return video_path


def _dispatch_message(cmsg: ChatMessage, is_group: bool):
    channel = WeworkChannel()
    if is_group:
        channel.handle_group(cmsg)
    else:
        channel.handle_single(cmsg)


def on_wework_message(wework_instance, message):
    """ntwork recv callback (registered from run.register_message_handlers)."""
    if "data" not in message:
        return None
    data = message["data"]
    conversation_id = data.get("conversation_id") or data.get("room_conversation_id")
    if not conversation_id:
        logger.debug("[Wework] message without conversation_id, skip")
        return None
    is_group = "R:" in conversation_id
    logger.info(
        f"[Wework] recv {'group' if is_group else 'single'} "
        f"type={message.get('type')} conv={conversation_id}"
    )
    try:
        cmsg = WeworkMessage(message, wework=wework_instance, is_group=is_group)
    except NotImplementedError as e:
        logger.debug(f"[Wework] skip message: {e}")
        return None
    except Exception as e:
        logger.error(f"[Wework] parse message failed: {e}", exc_info=True)
        return None
    delay = random.randint(1, 2)
    threading.Timer(delay, _dispatch_message, args=(cmsg, is_group)).start()
    return None


@singleton
class WeworkChannel(ChatChannel):
    NOT_SUPPORT_REPLYTYPE = []

    def __init__(self):
        super().__init__()
        self._stop_event = threading.Event()
        self._channel_ready = False
        self._sync_thread = None
        self.received_msgs = ExpiredDict(conf().get("expires_in_seconds", 3600))

    @property
    def login_status(self) -> str:
        if getattr(self, "user_id", None):
            return "logged_in"
        return "unknown"

    def startup(self):
        self._stop_event.clear()
        self._channel_ready = False
        self.notify_channel_status("starting")

        if ntwork is None:
            err = (
                "ntwork is not installed (requires Python 3.10 on Windows, not on PyPI). "
                "From repo root: pnpm run bootstrap:sidecar-wheels && pnpm run setup:channel-sidecar-dev, "
                "set CHANNEL_PYTHON_EXECUTABLE to py -3.10, then restart tauri dev. "
                "See channel_agent/README.md."
            )
            logger.error(f"[Wework] {err}")
            self.report_startup_error(err)
            return

        try:
            client = init_wework_client()
        except Exception as e:
            err = f"WeCom client init failed: {e}"
            logger.error(f"[Wework] {err}", exc_info=True)
            self.report_startup_error(err)
            return

        register_message_handlers(client)

        self.notify_channel_status("waiting_login")
        try:
            logger.info("[Wework] Opening WeCom desktop with smart=True")
            client.open(smart=True)
            logger.info("[Wework] Waiting for WeCom desktop login...")
            client.wait_login()
            login_info = client.get_login_info() or {}
            if not login_info.get("user_id"):
                raise RuntimeError("WeCom login finished but get_login_info returned empty user_id")
        except Exception as e:
            err = f"WeCom login failed: {e}"
            logger.error(f"[Wework] {err}")
            self.report_startup_error(err)
            return

        self.user_id = login_info.get("user_id", "")
        self.name = login_info.get("nickname") or login_info.get("username", "")
        logger.info(f"[Wework] Logged in user_id={self.user_id} name={self.name}")
        self.notify_channel_status(
            "logged_in",
            user_id=str(self.user_id),
            display_name=str(self.name),
        )
        self._channel_ready = True
        self.report_startup_success()
        logger.info("[Wework] Channel ready (desktop client / ntwork)")
        self._maybe_start_background_sync(client, force=False)
        run_until_stopped(self._stop_event)
        logger.info("[Wework] Event loop ended")

    def stop(self):
        logger.info("[Wework] stop() called")
        self._stop_event.set()
        if ntwork is not None:
            try:
                ntwork.exit_()
            except Exception as e:
                logger.warning(f"[Wework] ntwork.exit_ error: {e}")

    def _maybe_start_background_sync(self, client, force: bool) -> bool:
        """Start contact sync in background when needed."""
        if self._sync_thread is not None and self._sync_thread.is_alive():
            return False
        if not force and _contacts_sync_done(str(self.user_id or "")):
            logger.info("[Wework] Contact sync already completed for user_id=%s", self.user_id)
            return False

        self._sync_thread = threading.Thread(
            target=self._run_background_sync,
            args=(client, force),
            daemon=True,
            name="wework-contacts-sync",
        )
        self._sync_thread.start()
        return True

    def _run_background_sync(self, client, force: bool) -> None:
        """Fetch contacts and rooms in background without blocking channel readiness."""
        user_id = str(self.user_id or "")
        if not force and _contacts_sync_done(user_id):
            return
        try:
            self.notify_channel_status("syncing")
            logger.info("[Wework] Background contact sync started for user_id=%s", user_id)
            _write_sync_snapshot(client, _tmp_dir())
            _mark_contacts_sync_done(user_id)
            logger.info("[Wework] Background contact sync completed for user_id=%s", user_id)
        except Exception as e:
            logger.warning("[Wework] Background contact sync failed: %s", e, exc_info=True)

    def start_contacts_sync(self, force: bool = False) -> bool:
        """Start background contact sync on demand."""
        client = get_wework()
        if client is None:
            raise RuntimeError("ntwork not available")
        return self._maybe_start_background_sync(client, force=force)

    def _should_skip(self, cmsg: ChatMessage) -> bool:
        """Return True if this message should be ignored."""
        if not cmsg.msg_id:
            return True
        if cmsg.msg_id in self.received_msgs:
            logger.debug(f"[Wework] duplicate message, skip id={cmsg.msg_id}")
            return True
        self.received_msgs[cmsg.msg_id] = True
        if cmsg.ctype == ContextType.TEXT:
            if not cmsg.content.strip():
                return True
        return False

    @time_checker
    def handle_single(self, cmsg: ChatMessage):
        if self._should_skip(cmsg):
            return
        if self.user_id and cmsg.actual_user_id == self.user_id:
            logger.debug("[Wework] skip own single message")
            return
        if cmsg.ctype == ContextType.VOICE and not conf().get("speech_recognition"):
            return
        logger.info(f"[Wework] single msg ctype={cmsg.ctype}")
        context = self._compose_context(cmsg.ctype, cmsg.content, isgroup=False, msg=cmsg)
        if context:
            self.produce(context)

    @time_checker
    def handle_group(self, cmsg: ChatMessage):
        if self._should_skip(cmsg):
            return
        if self.user_id and cmsg.actual_user_id == self.user_id:
            logger.debug("[Wework] skip own group message")
            return
        if cmsg.ctype == ContextType.VOICE and not conf().get("speech_recognition"):
            return
        logger.info(
            f"[Wework] group msg group={cmsg.other_user_nickname!r} "
            f"from={cmsg.actual_user_nickname!r} at={cmsg.is_at} ctype={cmsg.ctype}"
        )
        context = self._compose_context(cmsg.ctype, cmsg.content, isgroup=True, msg=cmsg)
        if context:
            self.produce(context)
        else:
            logger.info(
                f"[Wework] group message not triggered "
                f"(check Rust channel trigger config / @bot prefix): "
                f"content={str(cmsg.content)[:80]!r}"
            )

    def send(self, reply: Reply, context: Context):
        client = get_wework()
        if client is None:
            logger.error("[Wework] cannot send: ntwork not available")
            return
        receiver = context["receiver"]
        actual_user_id = context["msg"].actual_user_id
        if reply.type in (ReplyType.TEXT, ReplyType.TEXT_):
            match = re.search(r"^@(.*?)\n", reply.content or "")
            if match and context.get("isgroup"):
                new_content = re.sub(r"^@(.*?)\n", "\n", reply.content, count=1)
                client.send_room_at_msg(receiver, new_content, [actual_user_id])
            else:
                client.send_text(receiver, reply.content)
            logger.info(f"[Wework] send text to {receiver}")
        elif reply.type in (ReplyType.ERROR, ReplyType.INFO):
            client.send_text(receiver, reply.content)
        elif reply.type == ReplyType.IMAGE:
            image_storage = reply.content
            image_storage.seek(0)
            with tempfile.NamedTemporaryFile(delete=False, suffix=".img") as temp:
                temp.write(image_storage.read())
                temp_path = temp.name
            try:
                client.send_image(receiver, temp_path)
            finally:
                try:
                    os.remove(temp_path)
                except OSError:
                    pass
        elif reply.type == ReplyType.IMAGE_URL:
            image_path = download_and_compress_image(reply.content, str(uuid.uuid4()))
            client.send_image(receiver, file_path=image_path)
        elif reply.type == ReplyType.VIDEO_URL:
            video_path = download_video(reply.content, str(uuid.uuid4()))
            if video_path is None:
                client.send_text(receiver, "抱歉，视频太大了！")
            else:
                client.send_video(receiver, video_path)
        elif reply.type in (ReplyType.VOICE, ReplyType.FILE):
            path = reply.content
            if not os.path.isabs(path):
                path = os.path.join(_tmp_dir(), os.path.basename(path))
            client.send_file(receiver, path)
            logger.info(f"[Wework] send file to {receiver}: {path}")
        else:
            logger.warning(f"[Wework] unsupported reply type: {reply.type}")
