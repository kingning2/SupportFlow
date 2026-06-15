# TypeScript 架构文档

## 定位

TypeScript 是桌面前端层，不是业务主后端。

## 负责内容

- 页面与交互
- 客户端状态管理（Redux）
- Tauri IPC 调用封装与事件订阅
- 共享类型消费（`contracts/`）
- 渲染渠道与 Agent 状态

## 不负责内容

- 渠道连接 / 断开 / 重启策略
- Agent 工具链与 LLM 编排
- 配置持久化与 Provider 凭据策略
- Python sidecar 生命周期

## 分层

```
apps/<channel>/          渠道私有页面、路由、特性 hooks
packages/ui/             通用组件、Agent 控制台壳、设计系统、标题栏
packages/shared/         IPC 桥接、contracts、Redux、渠道表单逻辑
```

### `apps/wework`

企业微信桌面前端：收件箱、账号切换、工作区布局、渠道导航。复用 `packages/ui/agent-console` 中的通用控制台视图。

### `packages/shared`

- `tauri-bridge/` — `invoke` 封装、command 枚举、事件名、窗口 API
- `contracts/` — typeshare 从 Rust 生成的 IPC 载荷类型
- `desktop-shell/` — Redux store、Tauri 事件 provider、初始化 guard
- `channel-core/` — 渠道连接表单、提示文案（无后端策略）

### `packages/ui`

- `agent-console/` — 对话、模型、技能、知识、通道、日志等通用视图
- `app-shell/` — 无边框主窗布局、channel bridge
- `design-system/` — CSS tokens、风味（wework）、作用域样式
- `title-bar/`、`modal/`、`license/` — 桌面壳能力

## 依赖方向

```
shared  →  ui  →  apps/*
```

禁止 `packages/shared` 依赖 `packages/ui`；禁止 `packages/*` 依赖 `apps/*`。

## 与 Rust 的边界

1. TS 不拥有后端策略。
2. TS 通过 `packages/shared/src/tauri-bridge/cmd/invoke.ts` 调用 Rust command。
3. TS 通过 `tauri-bridge` 枚举订阅 Rust 事件，用 Redux 或本地 state 渲染。
4. 新增 IPC 契约：改 Rust → 运行 `pnpm run generate:contracts` → 消费 `contracts/`。

## 全项目上下文

monorepo 总览见 [project-architecture.md](./project-architecture.md)、[project-folder-structure.md](./project-folder-structure.md)。
