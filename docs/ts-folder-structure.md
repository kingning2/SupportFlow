# TypeScript 文件夹结构

## 目标

TypeScript 只负责桌面前端界面、状态管理、以及调用 Rust IPC 的薄桥接。

## 核心目录

```
apps/wework/               个人企业微信桌面前端
packages/shared/           IPC、contracts、Redux、渠道表单
packages/ui/               组件库、Agent 控制台、设计系统
```

## `apps/wework/src/`

```
src/
├── main.tsx               Vite 入口
├── App.tsx
├── config/
│   ├── shell.ts           应用壳元数据
│   └── shell-accent.ts    主题强调色
└── features/wework/
    ├── app.tsx            特性根
    ├── router.tsx
    ├── accounts/          企微账号列表、切换、存储
    ├── constants/         wework-nav、示例 prompt
    ├── hooks/             use-wework-channel、use-wework-inbox…
    ├── inbox/             会话列表、消息线程、详情
    ├── layout/            sidebar、header、workspace-layout
    ├── styles/
    ├── types/
    └── views/             ai-chat、ai-config、knowledge、skills、mcp…
```

## `packages/shared/src/`

```
shared/src/
├── channel/               渠道类型定义
├── channel-core/          渠道表单字段、提示、草稿
├── contracts/             typeshare 生成（勿手改 contracts.ts）
├── desktop-shell/
│   ├── config/
│   ├── guards/global/     init-guard
│   ├── providers/         store、desktop-root、tauri-event
│   └── store/modules/     app、modal
├── tauri-bridge/
│   ├── cmd/               agent、license、window、wework-accounts…
│   ├── enums/             tauri-cmd、tauri-event、路由枚举
│   └── window/
└── utils/                 cn 等
```

## `packages/ui/src/`

```
ui/src/
├── agent-console/
│   ├── chat/              对话组件、MCP 侧栏
│   ├── layout/            控制台 app、sidebar、sessions
│   ├── views/             各管理页 + views/channels/
│   ├── hooks/
│   ├── constants/
│   └── lib/
├── app-shell/             DesktopAppLayout、desktop-app-root
├── design-system/         tokens、flavors、scopes
├── license/
├── modal/
├── title-bar/
└── *.tsx                  button、dialog、input 等基础组件
```

## 结构原则

1. 业务页面写在 `apps/*`。
2. 共用 IPC、类型、常量写在 `packages/shared/`。
3. 共用组件与控制台壳写在 `packages/ui/`。
4. TS 不直接承载后端策略，不重复实现 Rust 已有业务规则。
5. 新增 Tauri command：在 Rust `cmd/` 定义 → `generate:contracts` → 在 `tauri-bridge/cmd/` 增加封装。

## 全项目目录

见 [project-folder-structure.md](./project-folder-structure.md)。
