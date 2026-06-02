#!/usr/bin/env python3
"""One-shot migration: remove cow from paths and source in tauri-template."""
from __future__ import annotations

import os
import re
import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# Directory renames (old relative to ROOT)
DIR_RENAMES = [
    ("src-tauri/channel_agent", "src-tauri/channel_agent"),
    ("src-tauri/crates/supportflow-cli", "src-tauri/crates/supportflow-cli"),
]

# File renames (old relative to ROOT)
FILE_RENAMES = [
    ("src-tauri/src/context/channel_python_sidecar.rs", "src-tauri/src/context/channel_python_sidecar.rs"),
    ("src/cmd/channel-python-channels.ts", "src/cmd/channel-python-channels.ts"),
    ("src/enums/dev-channel.ts", "src/enums/dev-channel.ts"),
]

# Move markitdown requirements next to Python channel_agent
FILE_MOVES = [
    (
        "src-tauri/crates/agent/requirements-markitdown.txt",
        "src-tauri/channel_agent/requirements-markitdown.txt",
    ),
    (
        "src-tauri/crates/agent/resources/markitdown_convert.py",
        "src-tauri/channel_agent/scripts/markitdown_convert.py",
    ),
]

# Content replacements (order matters: longer tokens first)
REPLACEMENTS = [
    ("channel-sidecar-", "channel-sidecar-"),
    ("NEXT_PUBLIC_DEV_CHANNEL", "NEXT_PUBLIC_DEV_CHANNEL"),
    ("CHANNEL_MARKITDOWN_PYTHON", "CHANNEL_MARKITDOWN_PYTHON"),  # noop guard
    ("CHANNEL_MARKITDOWN_PYTHON", "CHANNEL_MARKITDOWN_PYTHON"),
    ("CHANNEL_SIDECAR_EXE", "CHANNEL_SIDECAR_EXE"),
    ("CHANNEL_START_DELAY_SECS", "CHANNEL_START_DELAY_SECS"),
    ("CHANNEL_BOOTSTRAP_PYTHON", "CHANNEL_BOOTSTRAP_PYTHON"),
    ("CHANNEL_SIDECAR_PYTHON", "CHANNEL_SIDECAR_PYTHON"),
    ("CHANNEL_PYTHON_EXECUTABLE", "CHANNEL_PYTHON_EXECUTABLE"),
    ("DEV_CHANNEL", "DEV_CHANNEL"),
    ("TAURI_CHANNEL_MODE", "TAURI_CHANNEL_MODE"),
    ("channel_python_sidecar", "channel_python_sidecar"),
    ("channel-python-channels", "channel-python-channels"),
    ("dev-channel", "dev-channel"),
    ("ChannelPythonSidecar", "ChannelPythonSidecar"),
    ("channel_python_channels_get", "channel_python_channels_get"),
    ("channel_python_channels_post", "channel_python_channels_post"),
    ("channel_python_channels_status", "channel_python_channels_status"),
    ("channel_console_api", "channel_console_api"),
    ("channelLangFromI18n", "channelLangFromI18n"),
    ("channelAction", "channelAction"),
    ("channelLoginStatus", "channelLoginStatus"),
    ("channelFieldValueString", "channelFieldValueString"),
    ("isChannelMaskedSecret", "isChannelMaskedSecret"),
    ("fetchChannels", "fetchChannels"),
    ("fetchChannelConsoleApi", "fetchChannelConsoleApi"),
    ("ChannelConsoleApiMethod", "ChannelConsoleApiMethod"),
    ("ChannelConsoleApiResponse", "ChannelConsoleApiResponse"),
    ("ChannelLocalized", "ChannelLocalized"),
    ("ChannelActionRequest", "ChannelActionRequest"),
    ("ChannelActionApiResponse", "ChannelActionApiResponse"),
    ("ChannelsApiResponse", "ChannelsApiResponse"),
    ("ChannelField", "ChannelField"),
    ("ChannelCatalogEntry", "ChannelCatalogEntry"),
    ("localizeChannelText", "localizeChannelText"),
    ("channel_agent.channel", "channel_agent.channel"),
    ("channel_agent", "channel_agent"),
    ("supportflow-cli", "supportflow-cli"),
    ("~/supportflow", "~/supportflow"),
    ("weixin_channel_credentials", "weixin_channel_credentials"),
    ("agent_workspace': '~/supportflow", "agent_workspace': '~/supportflow"),  # unlikely
    ("name = \"supportflow-cli\"", "name = \"supportflow-cli\""),
    ("default-run = \"cow\"", "default-run = \"sf\""),
    ("name = \"cow\"", "name = \"sf\""),
    ("Usage: sf ", "Usage: sf "),
    ("name = \"cow\"", "name = \"sf\""),
    ('"sf":', '"sf":'),
    ("-p supportflow-cli", "-p supportflow-cli"),
    ("bun run sf", "bun run sf"),
    ("SupportFlow Agent", "SupportFlow Agent"),
    ("//! SupportFlow CLI (`sf`)", "//! SupportFlow CLI (`sf`)"),
    ("Rust port of SupportFlow Agent", "Rust port of SupportFlow agent stack"),
]

TEXT_EXTENSIONS = {
    ".rs",
    ".ts",
    ".tsx",
    ".js",
    ".mjs",
    ".json",
    ".md",
    ".mdc",
    ".py",
    ".toml",
    ".lock",
    ".ps1",
    ".txt",
    ".yml",
    ".yaml",
    ".sh",
    ".gitignore",
    ".css",
    ".html",
}


def rename_path(old: Path, new: Path) -> None:
    if not old.exists():
        return
    if new.exists():
        raise FileExistsError(f"target exists: {new}")
    new.parent.mkdir(parents=True, exist_ok=True)
    old.rename(new)
    print(f"rename: {old.relative_to(ROOT)} -> {new.relative_to(ROOT)}")


def apply_content_replacements(text: str) -> str:
    out = text
    for old, new in REPLACEMENTS:
        if old == new:
            continue
        out = out.replace(old, new)
    # Remove accidental double replacements in enum names
    out = out.replace("ChannelCatalogEntry", "ChannelCatalogEntry")
    return out


def rewrite_file(path: Path) -> None:
    try:
        raw = path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        try:
            raw = path.read_text(encoding="utf-8-sig")
        except UnicodeDecodeError:
            return
    updated = apply_content_replacements(raw)
    if updated != raw:
        path.write_text(updated, encoding="utf-8", newline="\n")


SKIP_PARTS = frozenset({"node_modules", "target", ".git", ".next", "out", "build"})


def rename_files_with_cow_in_name() -> None:
    for path in list(ROOT.rglob("*")):
        if not path.is_file():
            continue
        if SKIP_PARTS.intersection(path.parts):
            continue
        if "cow" not in path.name.lower():
            continue
        if path.name == "migrate-remove-cow.py":
            continue
        new_name = path.name.replace("cow", "").replace("Cow", "")
        new_name = new_name.replace("--", "-").replace("__", "_")
        if new_name != path.name:
            target = path.with_name(new_name)
            if not target.exists():
                path.rename(target)
                print(f"file rename: {path.relative_to(ROOT)} -> {target.name}")


def main() -> None:
    os.chdir(ROOT)

    for old_rel, new_rel in DIR_RENAMES:
        rename_path(ROOT / old_rel, ROOT / new_rel)

    for old_rel, new_rel in FILE_RENAMES:
        rename_path(ROOT / old_rel, ROOT / new_rel)

    for old_rel, new_rel in FILE_MOVES:
        old = ROOT / old_rel
        new = ROOT / new_rel
        if old.is_file():
            new.parent.mkdir(parents=True, exist_ok=True)
            if new.exists():
                new.unlink()
            shutil.move(str(old), str(new))
            print(f"move: {old_rel} -> {new_rel}")

    rename_files_with_cow_in_name()

    for path in ROOT.rglob("*"):
        if not path.is_file():
            continue
        if path.suffix.lower() not in TEXT_EXTENSIONS and path.name not in (
            ".gitignore",
            "AGENTS.md",
        ):
            continue
        if SKIP_PARTS.intersection(path.parts):
            continue
        rewrite_file(path)

    print("migration complete")


if __name__ == "__main__":
    main()
