# encoding:utf-8
"""Global HTTP proxy control for ``requests`` (LLM APIs, downloads, etc.).

This module is tolerated in Python only because some sidecar-local SDK and
``requests`` calls still need proxy adaptation. New desktop-wide proxy policy,
configuration interpretation, and user-facing orchestration should live in
Rust, with Python consuming only the resolved runtime settings.

Config (``config.json``):

- ``use_proxy`` (bool, default ``false``): when ``false``, ignore ``HTTP_PROXY`` /
  ``HTTPS_PROXY`` and connect directly (fixes stale local proxy env).
- ``use_proxy`` (``true``): use ``proxy`` if set, otherwise system env vars.
- ``proxy`` (str): e.g. ``http://127.0.0.1:7890`` when ``use_proxy`` is ``true``.
"""

from __future__ import annotations

import os
from contextlib import contextmanager
from typing import Any, Dict, Iterator, Optional, Tuple

import requests

from common.log import logger
from config import conf

_session: Optional[requests.Session] = None


def use_proxy_enabled() -> bool:
    return bool(conf().get("use_proxy"))


def configured_proxy_url() -> str:
    return (conf().get("proxy") or "").strip()


def system_proxy_urls() -> Tuple[str, str]:
    http_p = (os.environ.get("HTTP_PROXY") or os.environ.get("http_proxy") or "").strip()
    https_p = (os.environ.get("HTTPS_PROXY") or os.environ.get("https_proxy") or "").strip()
    return http_p, https_p


def resolve_requests_proxies(explicit_proxy: Optional[str] = None) -> Optional[Dict[str, str]]:
    """Build ``requests`` ``proxies`` dict. ``None`` means no explicit proxy."""
    if explicit_proxy is not None:
        url = (explicit_proxy or "").strip()
        return {"http": url, "https": url} if url else None

    if not use_proxy_enabled():
        return None

    url = configured_proxy_url()
    if url:
        return {"http": url, "https": url}

    http_p, https_p = system_proxy_urls()
    if http_p or https_p:
        return {
            "http": http_p or https_p,
            "https": https_p or http_p,
        }
    return None


def requests_trust_env(explicit_proxy: Optional[str] = None) -> bool:
    """Whether ``requests`` may read proxy settings from the environment."""
    if explicit_proxy is not None:
        return False
    if not use_proxy_enabled():
        return False
    if configured_proxy_url():
        return False
    http_p, https_p = system_proxy_urls()
    return bool(http_p or https_p)


def describe_http_proxy() -> str:
    """Human-readable proxy mode for logs and UI."""
    if not use_proxy_enabled():
        http_p, https_p = system_proxy_urls()
        if http_p or https_p:
            ignored = https_p or http_p
            return (
                "disabled (use_proxy=false); ignoring system proxy "
                f"{ignored}"
            )
        return "disabled (use_proxy=false); direct connection"

    url = configured_proxy_url()
    if url:
        return f"enabled via config proxy={url}"

    http_p, https_p = system_proxy_urls()
    if http_p or https_p:
        return f"enabled via environment (HTTP={http_p or '-'}, HTTPS={https_p or '-'})"

    return "enabled (use_proxy=true) but no proxy URL configured; direct connection"


def log_http_proxy_settings() -> None:
    logger.info("[HTTP] Proxy mode: %s", describe_http_proxy())


_PROXY_ENV_KEYS = (
    "HTTP_PROXY",
    "HTTPS_PROXY",
    "http_proxy",
    "https_proxy",
    "ALL_PROXY",
    "all_proxy",
)


@contextmanager
def bypass_system_proxy() -> Iterator[None]:
    """When ``use_proxy`` is false, bypass WinHTTP/registry proxy for third-party SDKs.

    Raw ``requests`` calls (and some SDKs) still honor the OS proxy unless
    ``NO_PROXY=*`` is set or ``trust_env`` is false. Use around code paths that
    do not go through :func:`get_http_session`.
    """
    if use_proxy_enabled():
        yield
        return

    saved_proxy = {k: os.environ[k] for k in _PROXY_ENV_KEYS if k in os.environ}
    saved_no_proxy = os.environ.get("NO_PROXY")
    saved_no_proxy_lower = os.environ.get("no_proxy")
    try:
        for key in _PROXY_ENV_KEYS:
            os.environ.pop(key, None)
        os.environ["NO_PROXY"] = "*"
        os.environ["no_proxy"] = "*"
        yield
    finally:
        for key in _PROXY_ENV_KEYS:
            os.environ.pop(key, None)
        for key, value in saved_proxy.items():
            os.environ[key] = value
        if saved_no_proxy is None:
            os.environ.pop("NO_PROXY", None)
        else:
            os.environ["NO_PROXY"] = saved_no_proxy
        if saved_no_proxy_lower is None:
            os.environ.pop("no_proxy", None)
        else:
            os.environ["no_proxy"] = saved_no_proxy_lower


def reset_http_session() -> None:
    global _session
    if _session is not None:
        try:
            _session.close()
        except Exception:
            pass
    _session = None


def _configure_session(session: requests.Session, explicit_proxy: Optional[str] = None) -> None:
    session.trust_env = requests_trust_env(explicit_proxy)
    session.proxies.clear()
    proxies = resolve_requests_proxies(explicit_proxy)
    if proxies:
        session.proxies.update(proxies)


def get_http_session(explicit_proxy: Optional[str] = None) -> requests.Session:
    """Return a shared ``requests.Session`` with global proxy rules applied."""
    global _session
    if explicit_proxy is not None:
        session = requests.Session()
        _configure_session(session, explicit_proxy)
        return session
    if _session is None:
        _session = requests.Session()
        _configure_session(_session)
    else:
        _configure_session(_session)
    return _session


def http_request(method: str, url: str, explicit_proxy: Optional[str] = None, **kwargs: Any):
    """``requests.request`` honoring global proxy settings."""
    session = get_http_session(explicit_proxy)
    return session.request(method, url, **kwargs)


def http_get(url: str, **kwargs: Any):
    return http_request("GET", url, **kwargs)


def http_post(url: str, **kwargs: Any):
    return http_request("POST", url, **kwargs)
