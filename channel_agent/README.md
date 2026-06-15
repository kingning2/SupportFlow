# Channel Sidecar

Python sidecar for WeCom (`wework`) channel access. Rust owns desktop orchestration; Python only handles SDK login, messaging, and media download.

## Scope

- `channel/wework/`: WeCom desktop (`ntwork`)
- `bridge/`: Context/Reply types for Rust IPC
- `common/`, `config.py`: shared sidecar config and logging
- `scripts/markitdown_convert.py`: one-shot MarkItDown helper (invoked by Rust, not sidecar)

## Development

```bash
pnpm run setup:channel-sidecar-dev
pnpm run tauri:dev:wework
```

Manual run:

```bash
cd channel_agent
set CHANNEL_CONFIG_PATH=..\resources\config.json
py -3.10 -m channel
```

## Build

```bash
pnpm run bootstrap:sidecar-wheels
pnpm run build:channel-sidecar
```

`requirements-sidecar.txt` holds shared runtime deps; `requirements-wework.txt` adds `ntwork` / `pilk` for WeCom.
