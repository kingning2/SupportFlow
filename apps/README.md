# Frontend Monorepo

```
apps/
  full/      完整控制台
  wework/    企微独立应用（含 wework-page.tsx 等私有代码）
  wechat/    微信独立应用（含 wechat-page.tsx 等私有代码）

packages/
  shared/    公用逻辑
  ui/        公用 UI
```

## 原则

- **`packages/`** 只有 `shared` + `ui`，不放任何渠道包
- **渠道私有页面、accent、路由** 放在对应 `apps/<channel>/src/`

## 开发

```bash
bun install
bun run dev
bun run typecheck
bun run tauri:dev:wework
```

## 构建

| 命令                          | 产物               |
| ----------------------------- | ------------------ |
| `bun run build`               | `apps/full/out/`   |
| `bun run build:flavor:wework` | `apps/wework/out/` |
