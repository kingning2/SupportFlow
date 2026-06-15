# 项目文件夹结构

本文档描述 **SupportFlow monorepo** 的顶层目录与各子系统源码树。按语言编写的细节见 `rust-*`、`python-*`、`ts-*` 文档。

---

## 仓库根目录

```
tauri-template/                    # SupportFlow 主仓库
├── apps/                          # 前端应用（pnpm workspace）
├── packages/                      # 共享前端包（pnpm workspace）
├── src-tauri/                     # Rust 桌面后端（Tauri + sf CLI）
├── channel_agent/                 # Python 渠道 sidecar 源码
├── docs/                          # 架构、规范、文件夹结构文档
├── scripts/                       # 构建、契约生成、检查脚本
├── templates/                     # 新渠道应用脚手架
├── plan/                          # 结构重构 backlog、设计草案
├── subscription-activation-generator/  # 许可证生成工具（独立 Rust 小工具）
├── .agents/                       # AI 协作 agents / skills / rules
├── .cursor/                       # Cursor 规则与技能
├── .github/workflows/             # CI 工作流
├── .husky/                        # Git hooks
├── AGENTS.md                      # AI 协作入口（按文件类型引用 docs/）
├── README.md                      # 产品说明与快速开始
├── package.json                   # 根 package 脚本
├── pnpm-workspace.yaml            # workspace: packages/*, apps/*
├── turbo.json                     # Turborepo 任务编排
├── typeshare.toml                 # Rust ↔ TS 契约生成配置
└── .dependency-cruiser.cjs        # 前端依赖边界检查
```

---

## 前端 monorepo

### `apps/`

```
apps/
├── README.md
└── wework/                        # 个人企业微信桌面前端
    ├── src/
    │   ├── main.tsx               # Vite 入口
    │   ├── App.tsx
    │   ├── index.css
    │   ├── config/
    │   │   ├── shell.ts           # 应用壳配置
    │   │   └── shell-accent.ts    # 主题强调色
    │   └── features/wework/
    │       ├── app.tsx            # 特性根组件
    │       ├── page.tsx
    │       ├── router.tsx
    │       ├── accounts/          # 企微账号管理
    │       ├── constants/         # 导航、示例 prompt
    │       ├── hooks/             # 渠道、收件箱、活跃账号
    │       ├── inbox/             # 会话列表、消息线程
    │       ├── layout/            # 侧栏、顶栏、工作区布局
    │       ├── routes/
    │       ├── styles/
    │       ├── types/
    │       └── views/             # AI 对话、配置、知识、技能、MCP 等页
    ├── package.json
    └── vite.config.ts
```

**原则**：渠道私有页面、路由、特性代码留在 `apps/<channel>/`；不放入 `packages/`。

### `packages/`

```
packages/
├── README.md
├── shared/                        # @supportflow/shared
│   └── src/
│       ├── index.ts
│       ├── channel/               # 渠道类型
│       ├── channel-core/          # 渠道表单、提示、草稿
│       ├── contracts/             # typeshare 生成的 IPC 契约
│       ├── desktop-shell/         # Redux store、providers、init guard
│       │   ├── config/
│       │   ├── guards/
│       │   ├── providers/
│       │   └── store/modules/     # app、modal
│       ├── tauri-bridge/          # invoke 封装、枚举、事件
│       │   ├── cmd/               # agent、license、window、wework-accounts…
│       │   ├── enums/
│       │   └── window/
│       └── utils/
└── ui/                            # @supportflow/ui
    └── src/
        ├── agent-console/         # 通用 Agent 控制台
        │   ├── chat/              # 对话 UI、MCP 侧栏
        │   ├── constants/
        │   ├── hooks/
        │   ├── layout/            # 控制台布局、会话侧栏
        │   ├── lib/
        │   ├── shared/
        │   ├── styles/
        │   ├── types/
        │   └── views/             # 模型、技能、知识、通道、日志…
        ├── app-shell/             # 桌面主窗布局、channel bridge
        ├── design-system/         # tokens、flavors、scopes
        ├── layout/
        ├── license/               # 许可证门禁 UI
        ├── modal/                 # 子窗口 Modal
        ├── title-bar/             # 自定义标题栏
        └── *.tsx                  # 通用 shadcn/Semi 基础组件
```

**依赖方向**：`shared` → `ui` → `apps/*`

---

## Rust 后端 `src-tauri/`

```
src-tauri/
├── src/
│   ├── main.rs                    # 桌面二进制入口
│   ├── lib.rs                     # 模块声明、Tauri 应用构建
│   ├── contracts.rs
│   ├── bin/sf.rs                  # CLI 入口
│   ├── cmd/                       # Tauri commands（薄）
│   │   ├── agent_ipc.rs
│   │   ├── channel_inbox.rs
│   │   ├── license.rs
│   │   ├── log.rs
│   │   ├── wework_accounts.rs
│   │   └── window.rs
│   ├── context/                   # 状态与编排
│   │   ├── agent_runtime/       # Agent 运行时（控制台、会话、流、工作区）
│   │   ├── channel/             # 渠道 sidecar、收件箱、账号
│   │   ├── agent_runtime.rs
│   │   ├── license_store.rs
│   │   └── process_hub.rs
│   ├── services/
│   │   ├── agent/               # 工具链、rig、知识库、记忆、技能
│   │   │   ├── tools/           # read/write/bash/MCP/memory/…
│   │   │   ├── rig/
│   │   │   ├── knowledge/
│   │   │   ├── memory/
│   │   │   ├── skills/
│   │   │   ├── protocol/
│   │   │   ├── context/
│   │   │   └── workspace/
│   │   ├── bridge/              # AgentBridge、Bot 路由、配置同步
│   │   └── channel/
│   ├── config/                  # config.json、Provider 目录
│   ├── io/                        # 带日志文件 IO
│   ├── channel_runtime/           # 消息前缀/关键词/装饰（纯函数）
│   ├── process_runtime/           # 子进程、stdio NDJSON RPC
│   ├── python/                    # Python 互操作唯一入口
│   │   ├── sidecar/
│   │   ├── markitdown.rs
│   │   ├── paths.rs
│   │   └── paths_desktop.rs
│   ├── cli/                       # sf 子命令
│   ├── events/                    # 事件发射与载荷
│   └── utils/                     # 无 Store 通用工具
├── binaries/                      # channel-sidecar 构建产物
├── capabilities/                  # Tauri 权限能力
├── icons/
├── resources/                     # config、skills、公钥等
│   └── skills/                    # 内置 SKILL.md
├── scripts/
├── tests/
├── build.rs
├── Cargo.toml
├── tauri.conf.json
└── tauri.wework.conf.json
```

详见 [rust-folder-structure.md](./rust-folder-structure.md)。

---

## Python 渠道 `channel_agent/`

```
channel_agent/
├── channel/
│   ├── wework/                    # 企业微信 SDK（ntwork）
│   │   ├── run.py
│   │   ├── wework_channel.py
│   │   └── wework_message.py
│   ├── channel_manager.py         # sidecar 内渠道实例化
│   ├── rpc_handlers.py            # RPC 分发
│   ├── rust_ipc.py                # 双向 NDJSON RPC
│   ├── stdio_server.py            # sidecar 启动入口
│   ├── chat_channel.py
│   ├── chat_message.py
│   ├── __init__.py
│   └── __main__.py                # python -m channel
├── bridge/                        # 历史桥接层（迁移中，见 plan/）
│   ├── context.py
│   └── reply.py
├── common/                        # 最小通用工具（迁移中）
├── scripts/
│   └── markitdown_convert.py      # 单次 Markdown 转换（非 sidecar）
├── config.py
├── channel-sidecar-build.spec     # PyInstaller 规格
├── requirements-markitdown.txt
├── requirements-sidecar.txt
└── requirements-wework.txt
```

详见 [python-folder-structure.md](./python-folder-structure.md)。

---

## 工具与辅助目录

### `scripts/`

| 文件                          | 用途                    |
| ----------------------------- | ----------------------- |
| `build-flavor.mjs`            | 按风味构建前端          |
| `tauri-build-flavor.mjs`      | 按风味构建 Tauri        |
| `tauri-dev-channel.mjs`       | 渠道开发启动            |
| `generate-contracts.mjs`      | 运行 typeshare 生成契约 |
| `check-rust-architecture.ps1` | Rust 架构约束检查       |

### `templates/channel-app/`

新渠道前端脚手架（shell 配置、布局模板）。

### `plan/`

| 文件                                     | 用途                 |
| ---------------------------------------- | -------------------- |
| `project-structure-refactor-backlog.md`  | 结构治理任务清单     |
| `rust-sidecar-async-ipc-architecture.md` | sidecar IPC 设计草案 |

### `docs/`

| 文件                                    | 用途                       |
| --------------------------------------- | -------------------------- |
| `project-architecture.md`               | 全项目架构总览             |
| `project-folder-structure.md`           | 本文档                     |
| `rust-*.md` / `python-*.md` / `ts-*.md` | 分语言架构、目录、编码规范 |

### `subscription-activation-generator/`

独立的许可证激活码生成小工具（Rust），与主应用解耦。

---

## 写入规则速查

| 我要写…               | 放在哪里                                            |
| --------------------- | --------------------------------------------------- |
| 企业微信专属页面      | `apps/wework/src/features/wework/`                  |
| 通用控制台视图        | `packages/ui/src/agent-console/views/`              |
| IPC 封装 / 共享类型   | `packages/shared/src/tauri-bridge/` 或 `contracts/` |
| Tauri command         | `src-tauri/src/cmd/`                                |
| 跨 Webview 状态、编排 | `src-tauri/src/context/`                            |
| Agent / 渠道业务能力  | `src-tauri/src/services/`                           |
| 调 Python             | `src-tauri/src/python/`（唯一入口）                 |
| 企微 SDK 适配         | `channel_agent/channel/wework/`                     |
| 文档转 Markdown 脚本  | `channel_agent/scripts/markitdown_convert.py`       |
