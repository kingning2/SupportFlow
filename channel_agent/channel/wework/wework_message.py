"""
Parse ntwork / WeCom desktop client messages into ChatMessage.
"""

import datetime
import json
import os
import re
import time

from bridge.context import ContextType
from channel.chat_message import ChatMessage
from common.log import logger
from common.utils import expand_path
from config import conf

try:
    import pilk
except ImportError:
    pilk = None

try:
    from ntwork.const import send_type
except ImportError:
    send_type = None


def get_with_retry(get_func, max_retries=5, delay=5):
    retries = 0
    result = None
    while retries < max_retries:
        result = get_func()
        if result:
            break
        logger.warning(f"[Wework] get data failed, retry {retries + 1}/{max_retries}")
        retries += 1
        time.sleep(delay)
    return result


def _tmp_dir() -> str:
    ws_root = expand_path(conf().get("agent_workspace", "~/supportflow"))
    tmp_dir = os.path.join(ws_root, "tmp")
    os.makedirs(tmp_dir, exist_ok=True)
    return tmp_dir


def get_room_info(wework, conversation_id):
    rooms = wework.get_rooms()
    if not rooms or "room_list" not in rooms:
        logger.error(f"[Wework] get_rooms failed: {rooms}")
        return None
    time.sleep(1)
    for room in rooms["room_list"]:
        if room["conversation_id"] == conversation_id:
            return room
    return None


def cdn_download(wework, message, file_name):
    data = message["data"]
    aes_key = data["cdn"]["aes_key"]
    file_size = data["cdn"]["size"]
    save_path = os.path.join(_tmp_dir(), file_name)

    if "url" in data["cdn"] and "auth_key" in data["cdn"]:
        url = data["cdn"]["url"]
        auth_key = data["cdn"]["auth_key"]
        payload = {
            "url": url,
            "auth_key": auth_key,
            "aes_key": aes_key,
            "size": file_size,
            "save_path": save_path,
        }
        result = wework._WeWork__send_sync(send_type.MT_WXCDN_DOWNLOAD_MSG, payload)
    elif "file_id" in data["cdn"]:
        file_type = 2 if message["type"] == 11042 else 5
        file_id = data["cdn"]["file_id"]
        result = wework.c2c_cdn_download(file_id, aes_key, file_size, file_type, save_path)
    else:
        logger.error(f"[Wework] unknown CDN payload: {data}")
        return
    logger.debug(f"[Wework] CDN download result: {result}")


def c2c_download_and_convert(wework, message, file_name):
    if pilk is None:
        logger.error("[Wework] pilk not installed, cannot convert voice (pip install pilk)")
        return
    data = message["data"]
    aes_key = data["cdn"]["aes_key"]
    file_size = data["cdn"]["size"]
    file_id = data["cdn"]["file_id"]
    save_path = os.path.join(_tmp_dir(), file_name)
    result = wework.c2c_cdn_download(file_id, aes_key, file_size, 5, save_path)
    logger.debug(f"[Wework] c2c download: {result}")
    base_name, _ = os.path.splitext(save_path)
    wav_file = base_name + ".wav"
    pilk.silk_to_wav(save_path, wav_file, rate=24000)
    try:
        os.remove(save_path)
    except OSError:
        pass


class WeworkMessage(ChatMessage):
    def __init__(self, wework_msg, wework, is_group=False):
        try:
            super().__init__(wework_msg)
            data = wework_msg.get("data", {})
            self.msg_id = str(
                data.get("msgid")
                or data.get("msg_id")
                or f"{data.get('send_time')}_{data.get('sender')}_{wework_msg.get('type')}"
            )
            self.create_time = data.get("send_time")
            self.is_group = is_group
            self.wework = wework
            self.is_at = False

            if wework_msg["type"] == 11041:
                if any(
                    s in data.get("content", "")
                    for s in ("该消息类型暂不能展示", "不支持的消息类型")
                ):
                    raise NotImplementedError("unsupported placeholder text")
                self.ctype = ContextType.TEXT
                self.content = data["content"]
            elif wework_msg["type"] == 11044:
                file_name = datetime.datetime.now().strftime("%Y%m%d%H%M%S") + ".silk"
                wav_name = os.path.splitext(file_name)[0] + ".wav"
                self.ctype = ContextType.VOICE
                self.content = os.path.join(_tmp_dir(), wav_name)
                self._prepare_fn = lambda: c2c_download_and_convert(wework, wework_msg, file_name)
            elif wework_msg["type"] == 11042:
                file_name = datetime.datetime.now().strftime("%Y%m%d%H%M%S") + ".jpg"
                self.ctype = ContextType.IMAGE
                self.content = os.path.join(_tmp_dir(), file_name)
                self._prepare_fn = lambda: cdn_download(wework, wework_msg, file_name)
            elif wework_msg["type"] == 11045:
                file_name = datetime.datetime.now().strftime("%Y%m%d%H%M%S") + data["cdn"]["file_name"]
                self.ctype = ContextType.FILE
                self.content = os.path.join(_tmp_dir(), file_name)
                self._prepare_fn = lambda: cdn_download(wework, wework_msg, file_name)
            elif wework_msg["type"] == 11047:
                self.ctype = ContextType.SHARING
                self.content = data["url"]
            elif wework_msg["type"] == 11072:
                self.ctype = ContextType.JOIN_GROUP
                member_list = data["member_list"]
                self.actual_user_nickname = member_list[0]["name"]
                self.actual_user_id = member_list[0]["user_id"]
                self.content = f"{self.actual_user_nickname}加入了群聊！"
                self._refresh_room_members_cache(wework)
            else:
                raise NotImplementedError(
                    f"Unsupported wework message type: {wework_msg.get('type')}"
                )

            login_info = self.wework.get_login_info()
            nickname = (
                f"{login_info['username']}({login_info['nickname']})"
                if login_info.get("nickname")
                else login_info.get("username", "")
            )
            user_id = login_info["user_id"]
            sender_id = data.get("sender")
            conversation_id = data.get("conversation_id")
            sender_name = data.get("sender_name")

            self.from_user_id = user_id if sender_id == user_id else conversation_id
            self.from_user_nickname = nickname if sender_id == user_id else sender_name
            self.to_user_id = user_id
            self.to_user_nickname = nickname
            self.other_user_nickname = sender_name
            self.other_user_id = conversation_id
            self.actual_user_id = data.get("sender")
            self.actual_user_nickname = sender_name

            if self.is_group:
                conversation_id = data.get("conversation_id") or data.get("room_conversation_id")
                self.other_user_id = conversation_id
                if conversation_id:
                    room_info = get_room_info(wework=wework, conversation_id=conversation_id)
                    room_name = room_info.get("nickname") if room_info else None
                    self.other_user_nickname = room_name
                    self.from_user_nickname = room_name
                    at_list = [a.get("nickname") for a in data.get("at_list", [])]
                    self.is_at = (
                        nickname in at_list
                        or login_info.get("nickname") in at_list
                        or login_info.get("username") in at_list
                    )
                    content = data.get("content", "")
                    pattern = f"@{re.escape(nickname)}(\u2005|\u0020)"
                    if re.search(pattern, content):
                        self.is_at = True
                    self.at_list = at_list
                    if self.ctype != ContextType.JOIN_GROUP:
                        self.actual_user_nickname = sender_name
                else:
                    logger.error("[Wework] group message missing conversation_id")

            logger.debug(f"[Wework] parsed message id={self.msg_id} ctype={self.ctype}")
        except Exception as e:
            logger.error(f"[Wework] WeworkMessage init error: {e}")
            raise

    @staticmethod
    def _refresh_room_members_cache(wework):
        directory = _tmp_dir()
        rooms = get_with_retry(wework.get_rooms)
        if not rooms:
            logger.error("[Wework] failed to refresh room list after join")
            return
        result = {}
        for room in rooms["room_list"]:
            room_wxid = room["conversation_id"]
            result[room_wxid] = wework.get_room_members(room_wxid)
        path = os.path.join(directory, "wework_room_members.json")
        with open(path, "w", encoding="utf-8") as f:
            json.dump(result, f, ensure_ascii=False, indent=4)
        logger.info("[Wework] room members cache updated")
