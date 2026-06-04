# Frontend Monorepo

```
apps/
  full/      完整控制台
  wework/    企微独立应用
  wechat/    微信独立应用

packages/
  shared/    公用逻辑
  ui/        公用 UI
```

## 原则

- **`packages/`** 只有 `shared` + `ui`，不放任何渠道包
- **渠道私有页面、accent、路由** 放在对应 `apps/<channel>/src/`

## 开发

```bash
pnpm install
pnpm run dev
pnpm run typecheck
pnpm run tauri:dev:wework
pnpm run create:channel -- --name dingtalk --title "SupportFlow · 钉钉"
```

## 构建

| 命令                           | 产物               |
| ------------------------------ | ------------------ |
| `pnpm run build`               | `apps/full/out/`   |
| `pnpm run build:flavor:wework` | `apps/wework/out/` |

## 新平台脚手架

新增平台不要再复制现有 app 目录，统一使用脚手架：

```bash
pnpm run create:channel -- --name dingtalk --title "SupportFlow · 钉钉"
```

生成结果位于 `apps/<name>/`，默认骨架为：

```text
src/
  app/
  assets/
  components/
  config/
  features/<name>/
```

约定：

- `app/` 只放 `layout.tsx` 和 `page.tsx`
- `config/` 放标题栏 accent、壳层 className
- `features/<name>/` 放平台私有页面和逻辑
- `components/` 放 app 私有复用组件
- 共享逻辑继续放 `packages/shared`，共享 UI 继续放 `packages/ui`

`apps/full` 也遵循同样思路：`app/` 保留路由入口，完整控制台与 modal 私有实现下沉到 `features/full/`。
