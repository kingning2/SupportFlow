# encoding:utf-8
"""Unified channel adapter contract (aligned with Rust `services/channel`)."""

from __future__ import annotations

from abc import ABC, abstractmethod

from bridge.context import Context
from bridge.reply import Reply


class ChannelAdapter(ABC):
    """Minimal sidecar adapter surface — SDK glue only; orchestration stays in Rust."""

    channel_type: str = ""
    NOT_SUPPORT_REPLYTYPE: list = []
    name: str | None = None
    user_id: str | None = None

    @abstractmethod
    def startup(self) -> None:
        """Boot SDK client and block until stop."""

    @abstractmethod
    def send(self, reply: Reply, context: Context) -> None:
        """Deliver one outbound message to the channel SDK."""

    def stop(self) -> None:
        """Release SDK resources."""

    def health(self) -> dict:
        """Sidecar health probe (optional)."""
        return {"channel": self.channel_type, "status": "ok"}
