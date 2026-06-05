# Frontend Monorepo

```text
apps/
  full/      Full desktop console
  wework/    Personal WeCom desktop app
  wechat/    Personal WeChat desktop app

packages/
  shared/    Shared logic and Tauri bridge
  ui/        Shared UI components
```

## Rules

- `packages/` only keeps `shared` and `ui`.
- Channel-specific pages, config, and feature code stay under `apps/wework` and `apps/wechat`.
- `apps/full` remains the shared desktop console shell.

## Development

```bash
pnpm install
pnpm run dev
pnpm run typecheck
pnpm run tauri:dev:wework
pnpm run tauri:dev:wechat
```

## Build

| Command                        | Output             |
| ------------------------------ | ------------------ |
| `pnpm run build`               | `apps/full/out/`   |
| `pnpm run build:flavor:wework` | `apps/wework/out/` |
| `pnpm run build:flavor:wechat` | `apps/wechat/out/` |
