# encoding:utf-8
"""Bridge → Rust AgentRuntime (no Python models)."""

from __future__ import annotations

from enum import Enum

from bridge.context import Context
from bridge.reply import Reply, ReplyType
from channel.rust_ipc import call_rust
from common.log import logger
from common.singleton import singleton


def _json_safe(val):
    """Values passed to Rust NDJSON RPC must be JSON-serializable."""
    if val is None:
        return None
    if isinstance(val, Enum):
        return val.name
    return val


def _context_params(query: str, context: Context | None, *, agent: bool, clear_history: bool = False) -> dict:
    params: dict = {"query": query, "agent": agent, "clear_history": clear_history}
    if not context:
        return params
    for key in (
        "session_id",
        "channel_type",
        "receiver",
        "isgroup",
        "group_name",
        "msg",
    ):
        if key in context:
            val = context[key]
            if key == "msg" and val is not None:
                params["msg_type"] = _json_safe(getattr(val, "ctype", None))
                params["from_user_id"] = getattr(val, "from_user_id", None)
            else:
                params[key] = _json_safe(val)
    return params


@singleton
class Bridge:
    def fetch_reply_content(self, query: str, context: Context | None = None) -> Reply:
        try:
            result = call_rust("agent.reply", _context_params(query, context, agent=False))
            content = result.get("content") or ""
            return Reply(ReplyType.TEXT, content)
        except Exception as e:
            logger.error("[Bridge] Rust reply failed: %s", e)
            return Reply(ReplyType.ERROR, str(e))

    def fetch_agent_reply(
        self,
        query: str,
        context: Context | None = None,
        on_event=None,
        clear_history: bool = False,
    ) -> Reply:
        if on_event:
            logger.debug("[Bridge] on_event streaming not forwarded in Tauri sidecar yet")
        try:
            result = call_rust(
                "agent.reply",
                _context_params(query, context, agent=True, clear_history=clear_history),
            )
            content = result.get("content") or ""
            return Reply(ReplyType.TEXT, content)
        except Exception as e:
            logger.error("[Bridge] Rust agent reply failed: %s", e)
            return Reply(ReplyType.ERROR, str(e))

    def fetch_voice_to_text(self, voice_file) -> Reply:
        return Reply(ReplyType.ERROR, "voice_to_text: use Rust agent (not implemented in sidecar)")

    def fetch_text_to_voice(self, text) -> Reply:
        return Reply(ReplyType.ERROR, "text_to_voice: use Rust agent (not implemented in sidecar)")

    def fetch_translate(self, text, from_lang="", to_lang="en") -> Reply:
        return Reply(ReplyType.TEXT, text)
