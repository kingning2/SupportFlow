import os
import time
from enum import Enum

from bridge.context import Context, ContextType
from bridge.reply import Reply, ReplyType
from channel.rust_ipc import call_rust, notify_rust
from common.log import logger
from config import conf


def _json_safe(val):
    """将传给 Rust 的值转换为可 JSON 序列化的形式。"""
    if val is None:
        return None
    if isinstance(val, Enum):
        return val.name
    return val


def _context_params(query: str, context: Context | None, clear_history: bool = False) -> dict:
    """把 Python 侧上下文对象整理为 Rust `agent.reply` 所需参数。"""
    params: dict = {"query": query, "clear_history": clear_history}
    if not context:
        return params
    for key in ("session_id", "channel_type", "receiver", "isgroup", "group_name", "msg"):
        if key not in context:
            continue
        val = context[key]
        if key == "msg" and val is not None:
            params["msg_type"] = _json_safe(getattr(val, "ctype", None))
            params["from_user_id"] = getattr(val, "from_user_id", None)
        else:
            params[key] = _json_safe(val)
    return params


def _reply_type_from_rust(name: str | None) -> ReplyType:
    """把 Rust 返回的字符串类型映射为 Python 侧回复类型。"""
    mapping = {
        "TEXT": ReplyType.TEXT,
        "VOICE": ReplyType.VOICE,
        "IMAGE": ReplyType.IMAGE,
        "IMAGE_URL": ReplyType.IMAGE_URL,
        "VIDEO_URL": ReplyType.VIDEO_URL,
        "FILE": ReplyType.FILE,
        "INFO": ReplyType.INFO,
        "ERROR": ReplyType.ERROR,
        "TEXT_FORCE": ReplyType.TEXT_,
    }
    if not name:
        return ReplyType.TEXT
    return mapping.get(name.upper(), ReplyType.TEXT)


def _reply_from_rust_result(result: dict) -> Reply:
    """把 Rust `agent.reply` 结果转换为 Python 回复对象。"""
    content = result.get("content") or ""
    ty = _reply_type_from_rust(result.get("reply_type"))
    reply = Reply(ty, content)
    text_content = result.get("text_content")
    if text_content:
        reply.text_content = text_content
    file_name = result.get("file_name")
    if file_name:
        reply.file_name = file_name
    return reply


class ChatChannel:
    """通道适配壳，公共消息编排逻辑由 Rust 提供。"""

    channel_type = ""
    NOT_SUPPORT_REPLYTYPE = [ReplyType.VOICE, ReplyType.IMAGE]
    name = None
    user_id = None

    def __init__(self):
        """初始化通道公共运行态。"""
        import threading

        self._startup_event = threading.Event()
        self._startup_error = None
        self.cloud_mode = False

    def startup(self):
        """启动通道，由具体通道实现。"""
        raise NotImplementedError

    def notify_channel_status(self, phase: str, **extra) -> None:
        """向 Rust 推送通道生命周期状态。"""
        if not self.channel_type:
            return
        try:
            params = {"channel": self.channel_type, "phase": phase, **extra}
            notify_rust("channel.notify", params)
        except Exception:
            pass

    def report_startup_success(self):
        """标记启动成功并广播 `ready` 状态。"""
        self._startup_error = None
        self._startup_event.set()
        self.notify_channel_status("ready")

    def report_startup_error(self, error: str):
        """标记启动失败并广播错误状态。"""
        self._startup_error = error
        self._startup_event.set()
        self.notify_channel_status("error", message=error)

    def wait_startup(self, timeout: float = 3) -> (bool, str):
        """等待启动结果，返回 `(是否成功, 错误信息)`。"""
        ready = self._startup_event.wait(timeout=timeout)
        if not ready:
            return True, ""
        if self._startup_error:
            return False, self._startup_error
        return True, ""

    def stop(self):
        """停止通道，由具体通道按需覆盖。"""
        pass

    def send(self, reply: Reply, context: Context):
        """发送消息，由具体通道实现。"""
        raise NotImplementedError

    def _build_reply_content(
        self,
        query: str,
        context: Context | None = None,
        clear_history: bool = False,
    ) -> Reply:
        """通过 Rust AgentRuntime 生成回复内容。"""
        result = call_rust("agent.reply", _context_params(query, context, clear_history))
        return _reply_from_rust_result(result)

    def _build_voice_to_text(self, voice_file) -> Reply:
        """语音转文本当前由 Rust 负责，Python 侧仅返回占位错误。"""
        return Reply(ReplyType.ERROR, "voice_to_text: use Rust agent (not implemented in sidecar)")

    def _build_text_to_voice(self, text) -> Reply:
        """文本转语音当前由 Rust 负责，Python 侧仅返回占位错误。"""
        return Reply(ReplyType.ERROR, "text_to_voice: use Rust agent (not implemented in sidecar)")

    def _compose_context(self, ctype: ContextType, content, **kwargs):
        """构造发送给 Rust 的标准上下文对象。"""
        context = Context(ctype, content)
        context.kwargs = kwargs
        context["channel_type"] = context.get("channel_type", self.channel_type)
        context["origin_ctype"] = context.get("origin_ctype", ctype)

        cmsg = context.get("msg")
        if cmsg and "receiver" not in context:
            context["receiver"] = cmsg.other_user_id
        if cmsg and "session_id" not in context:
            if context.get("isgroup", False):
                if conf().get("group_shared_session", True):
                    context["session_id"] = cmsg.other_user_id
                else:
                    context["session_id"] = cmsg.actual_user_id
            else:
                context["session_id"] = cmsg.other_user_id
        return context

    def _rust_process_context(self, context: Context):
        """调用 Rust 判断当前消息是否应触发处理。"""
        cfg = {
            "group_chat_prefix": conf().get("group_chat_prefix", []),
            "group_chat_keyword": conf().get("group_chat_keyword", []),
            "single_chat_prefix": conf().get("single_chat_prefix", [""]),
            "group_chat_reply_prefix": conf().get("group_chat_reply_prefix", ""),
            "group_chat_reply_suffix": conf().get("group_chat_reply_suffix", ""),
            "single_chat_reply_prefix": conf().get("single_chat_reply_prefix", ""),
            "single_chat_reply_suffix": conf().get("single_chat_reply_suffix", ""),
            "image_create_prefix": conf().get("image_create_prefix", []),
        }
        is_group = bool(context.get("isgroup", False))
        no_need_at = bool(context.get("no_need_at", False))
        if is_group and conf().get("group_at_off", False):
            no_need_at = True
        payload = {
            "context": {
                "channel_type": context.get("channel_type", ""),
                "is_group": is_group,
                "content": context.content or "",
                "actual_user_nickname": getattr(context.get("msg"), "actual_user_nickname", None),
                "no_need_at": no_need_at,
            },
            "config": cfg,
        }
        try:
            return call_rust("channel.process", payload)
        except Exception as e:
            logger.debug("[chat_channel] channel.process fallback to python: %s", e)
            return None

    def _rust_decorate_text(self, text: str, meta: dict):
        """调用 Rust 对回复文本做装饰。"""
        try:
            result = call_rust("channel.decorate_text", {"text": text, "meta": meta})
            return result.get("text", text)
        except Exception as e:
            logger.debug("[chat_channel] channel.decorate_text fallback to python: %s", e)
            return text

    def _rust_extract_media(self, text: str, limit: int = 5):
        """调用 Rust 从文本中提取图片或视频资源链接。"""
        try:
            result = call_rust("channel.extract_media", {"text": text, "limit": limit})
            items = result.get("items", [])
            if isinstance(items, list):
                return [
                    (i.get("url", ""), i.get("kind", "image"))
                    for i in items
                    if isinstance(i, dict)
                ]
            return []
        except Exception as e:
            logger.debug("[chat_channel] channel.extract_media fallback to python: %s", e)
            return []

    def _handle(self, context: Context):
        """处理一条通道消息并通过 Rust 生成回复。"""
        if context is None or not context.content:
            return
        rust_processed = self._rust_process_context(context)
        if rust_processed is None or not rust_processed.get("should_handle", False):
            return
        context.content = rust_processed.get("normalized_content", context.content)
        context["_reply_prefix"] = rust_processed.get("reply_prefix", "")
        context["_reply_suffix"] = rust_processed.get("reply_suffix", "")
        context["_mention_prefix"] = rust_processed.get("mention_prefix", "")
        logger.debug("[chat_channel] handling context: {}".format(context))
        if context.type == ContextType.VOICE:
            cmsg = context.get("msg")
            if cmsg:
                cmsg.prepare()
            reply = self._build_voice_to_text(context.content)
            if reply.type == ReplyType.TEXT:
                new_context = self._compose_context(ContextType.TEXT, reply.content, **context.kwargs)
                if new_context:
                    self._handle(new_context)
            return
        if context.type not in (ContextType.TEXT, ContextType.IMAGE_CREATE):
            return
        reply = self._build_reply_content(context.content, context)

        if not reply:
            return
        if reply.type == ReplyType.ERROR:
            logger.error("[chat_channel] reply error (not sent to channel): %s", reply.content)
            return
        if not reply.content:
            return
        self._send_reply(context, reply)

    def _send_reply(self, context: Context, reply: Reply):
        """把 Rust 回复结果整理后发送到具体通道。"""
        if reply and reply.type:
            logger.debug("[chat_channel] sending reply: {}, context: {}".format(reply, context))
            if reply.type == ReplyType.TEXT:
                raw_text = reply.content
                if context.get("isgroup", False):
                    meta = {
                        "should_handle": True,
                        "normalized_content": context.content or "",
                        "reply_prefix": context.get("_reply_prefix", conf().get("group_chat_reply_prefix", "")),
                        "reply_suffix": context.get("_reply_suffix", conf().get("group_chat_reply_suffix", "")),
                        "mention_prefix": context.get("_mention_prefix", ""),
                    }
                else:
                    meta = {
                        "should_handle": True,
                        "normalized_content": context.content or "",
                        "reply_prefix": context.get("_reply_prefix", conf().get("single_chat_reply_prefix", "")),
                        "reply_suffix": context.get("_reply_suffix", conf().get("single_chat_reply_suffix", "")),
                        "mention_prefix": "",
                    }
                reply.content = self._rust_decorate_text(raw_text, meta)
                self._extract_and_send_images(reply, context)
                return
            if reply.type in (ReplyType.ERROR, ReplyType.INFO):
                reply.content = "[" + str(reply.type) + "]\n" + reply.content
            self._send(reply, context)

    def _extract_and_send_images(self, reply: Reply, context: Context):
        """从回复文本里提取媒体资源并按通道能力顺序发送。"""
        content = reply.content
        media_items = self._rust_extract_media(content, limit=5)

        if media_items:
            logger.info(f"[chat_channel] Extracted {len(media_items)} media item(s) from reply")
            logger.info(f"[chat_channel] Sending text content before media: {reply.content[:100]}...")
            self._send(reply, context)
            logger.info(f"[chat_channel] Text sent, now sending {len(media_items)} media item(s)")

            for i, (url, media_type) in enumerate(media_items):
                try:
                    if url.startswith(("http://", "https://")):
                        if media_type == "video":
                            media_reply = Reply(ReplyType.FILE, url)
                            media_reply.file_name = os.path.basename(url)
                        else:
                            media_reply = Reply(ReplyType.IMAGE_URL, url)
                    elif os.path.exists(url):
                        if media_type == "video":
                            media_reply = Reply(ReplyType.FILE, f"file://{url}")
                            media_reply.file_name = os.path.basename(url)
                        else:
                            media_reply = Reply(ReplyType.IMAGE_URL, f"file://{url}")
                    else:
                        logger.warning(f"[chat_channel] Media file not found or invalid URL: {url}")
                        continue

                    if i > 0:
                        time.sleep(0.5)
                    self._send(media_reply, context)
                    logger.info(f"[chat_channel] Sent {media_type} {i + 1}/{len(media_items)}: {url[:50]}...")
                except Exception as e:
                    logger.error(f"[chat_channel] Failed to send {media_type} {url}: {e}")
        else:
            self._send(reply, context)

    def _send(self, reply: Reply, context: Context, retry_cnt=0):
        """执行发送并在可重试错误场景下进行有限重试。"""
        try:
            self.send(reply, context)
        except Exception as e:
            logger.error("[chat_channel] sendMsg error: {}".format(str(e)))
            if isinstance(e, NotImplementedError):
                return
            logger.exception(e)
            if retry_cnt < 2:
                time.sleep(3 + 3 * retry_cnt)
                self._send(reply, context, retry_cnt + 1)

    def produce(self, context: Context):
        """对外暴露统一消息处理入口。"""
        self._handle(context)
