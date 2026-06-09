# Frontend Monorepo

```text
apps/
  wework/    Personal WeCom desktop app

packages/
  shared/    Shared logic and Tauri bridge
  ui/        Shared UI components
```

## Rules

- `packages/` only keeps `shared` and `ui`.
- Channel-specific pages, config, and feature code stay under `apps/wework`.

## Development

```bash
pnpm install
pnpm run dev
pnpm run typecheck
pnpm run tauri:dev:wework
```

## Build

| Command                        | Output             |
| ------------------------------ | ------------------ |
| `pnpm run build`               | `apps/wework/out/` |
| `pnpm run build:flavor:wework` | `apps/wework/out/` |
