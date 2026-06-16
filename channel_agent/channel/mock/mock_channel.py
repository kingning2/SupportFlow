# encoding:utf-8
"""Mock channel adapter for spike / dev testing."""

from __future__ import annotations

import threading
import time
import uuid

from bridge.context import Context, ContextType
from bridge.reply import Reply, ReplyType
from channel.chat_channel import ChatChannel
from channel.chat_message import ChatMessage
from common.log import logger
from config import conf


class MockChannel(ChatChannel):
    """Simulated channel: auto-ready and injects one inbound message."""

    def startup(self) -> None:
        self.notify_channel_status("starting")
        self.user_id = "mock-user"
        self.name = "Mock User"
        self.notify_channel_status(
            "logged_in",
            user_id=self.user_id,
            display_name=self.name,
        )
        self.report_startup_success()
        logger.info("[Mock] channel ready")

        def inject_demo_message() -> None:
            time.sleep(float(conf().get("mock_inbound_delay_seconds", 0.5)))
            if getattr(self, "_stop_event", None) and self._stop_event.is_set():
                return
            cmsg = ChatMessage()
            cmsg.msg_id = str(uuid.uuid4())
            cmsg.ctype = ContextType.TEXT
            cmsg.content = conf().get(
                "mock_inbound_text", "你好，这是一条 mock 渠道测试消息。"
            )
            cmsg.other_user_id = "mock-conv-1"
            cmsg.other_user_nickname = "Mock Customer"
            cmsg.actual_user_id = "mock-customer"
            cmsg.actual_user_nickname = "Mock Customer"
            cmsg.create_time = int(time.time())
            context = self._compose_context(
                ContextType.TEXT, cmsg.content, isgroup=False, msg=cmsg
            )
            if context:
                self.produce(context)

        threading.Thread(target=inject_demo_message, daemon=True).start()
        stop = threading.Event()
        self._stop_event = stop
        while not stop.wait(1.0):
            pass

    def stop(self) -> None:
        if hasattr(self, "_stop_event"):
            self._stop_event.set()

    def send(self, reply: Reply, context: Context) -> None:
        content = reply.content or ""
        logger.info("[Mock] send to %s: %s", context.get("receiver"), content[:200])
