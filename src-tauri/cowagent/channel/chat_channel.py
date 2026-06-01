import os
import time

from bridge.context import Context, ContextType
from bridge.reply import Reply, ReplyType
from channel.channel import Channel
from channel.rust_ipc import call_rust
from common.log import logger
from config import conf


class ChatChannel(Channel):
    """Channel adapter shell. Channel-agnostic logic lives in Rust."""

    name = None
    user_id = None

    def _compose_context(self, ctype: ContextType, content, **kwargs):
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
        try:
            result = call_rust("channel.decorate_text", {"text": text, "meta": meta})
            return result.get("text", text)
        except Exception as e:
            logger.debug("[chat_channel] channel.decorate_text fallback to python: %s", e)
            return text

    def _rust_extract_media(self, text: str, limit: int = 5):
        try:
            result = call_rust("channel.extract_media", {"text": text, "limit": limit})
            items = result.get("items", [])
            if isinstance(items, list):
                return [(i.get("url", ""), i.get("kind", "image")) for i in items if isinstance(i, dict)]
            return []
        except Exception as e:
            logger.debug("[chat_channel] channel.extract_media fallback to python: %s", e)
            return []

    def _handle(self, context: Context):
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
            reply = super().build_voice_to_text(context.content)
            if reply.type == ReplyType.TEXT:
                new_context = self._compose_context(ContextType.TEXT, reply.content, **context.kwargs)
                if new_context:
                    self._handle(new_context)
            return
        if context.type not in (ContextType.TEXT, ContextType.IMAGE_CREATE):
            return
        reply = super().build_reply_content(context.content, context)

        if not reply:
            return
        if reply.type == ReplyType.ERROR:
            logger.error("[chat_channel] reply error (not sent to channel): %s", reply.content)
            return
        if not reply.content:
            return
        self._send_reply(context, reply)

    def _send_reply(self, context: Context, reply: Reply):
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
                if context.get("channel_type") != "web":
                    self._extract_and_send_images(reply, context)
                else:
                    self._send(reply, context)
                return
            if reply.type in (ReplyType.ERROR, ReplyType.INFO):
                reply.content = "[" + str(reply.type) + "]\n" + reply.content
            self._send(reply, context)
    
    def _extract_and_send_images(self, reply: Reply, context: Context):
        """
        从文本回复中提取图片/视频URL并单独发送
        支持格式：[图片: /path/to/image.png], [视频: /path/to/video.mp4], ![](url), <img src="url">
        最多发送5个媒体文件
        """
        content = reply.content
        media_items = self._rust_extract_media(content, limit=5)  # [(url, type), ...]
        
        if media_items:
            logger.info(f"[chat_channel] Extracted {len(media_items)} media item(s) from reply")
            
            # Send text first (the frontend will embed video players via renderMarkdown).
            logger.info(f"[chat_channel] Sending text content before media: {reply.content[:100]}...")
            self._send(reply, context)
            logger.info(f"[chat_channel] Text sent, now sending {len(media_items)} media item(s)")
            
            for i, (url, media_type) in enumerate(media_items):
                try:
                    # Determine whether it is a remote URL or a local file.
                    if url.startswith(('http://', 'https://')):
                        if media_type == 'video':
                            media_reply = Reply(ReplyType.FILE, url)
                            media_reply.file_name = os.path.basename(url)
                        else:
                            media_reply = Reply(ReplyType.IMAGE_URL, url)
                    elif os.path.exists(url):
                        if media_type == 'video':
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
                    logger.info(f"[chat_channel] Sent {media_type} {i+1}/{len(media_items)}: {url[:50]}...")
                    
                except Exception as e:
                    logger.error(f"[chat_channel] Failed to send {media_type} {url}: {e}")
        else:
            # 没有媒体文件，正常发送文本
                self._send(reply, context)

    def _send(self, reply: Reply, context: Context, retry_cnt=0):
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
        self._handle(context)
