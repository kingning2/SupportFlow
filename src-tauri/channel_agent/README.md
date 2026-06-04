# Channel Sidecar

Python sidecar for channel access. Rust stays frontend-facing and orchestrates the app; Python only handles channel connectivity for the supported personal channels.

## Scope

- `channel/wechat/`: personal WeChat (`wx`)
- `channel/wework/`: personal WeCom desktop (`wework`)
- `bridge/`: Python to Rust bridge
- `common/`, `config.py`, `lib/`: shared sidecar config, logging, and vendored libraries

## Development

```bash
pnpm run setup:channel-sidecar-dev
pnpm run tauri:dev:wework
pnpm run tauri:dev:wechat
```

Manual run:

```bash
cd src-tauri/channel_agent
set CHANNEL_CONFIG_PATH=..\resources\config.json
py -3.10 -m channel
```

## Build

```bash
pnpm run bootstrap:sidecar-wheels
pnpm run build:channel-sidecar
```

`requirements-sidecar.txt` now only contains shared runtime dependencies for the retained `wx` and `wework` channels. `requirements-wework.txt` handles the additional `ntwork` path for WeCom.
