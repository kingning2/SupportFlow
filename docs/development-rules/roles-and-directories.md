# 角色与目录地图

新人或协作时，先确认**自己的职责**，再对照下表找「主要改哪里 / 少碰哪里」。  
技术调用链见 [architecture.md](../../.agents/skills/tauri-next-coding-notes/architecture.md)。

---

## 按角色：你该动哪些目录

### 前端工程师（页面 / 组件 / 状态）

| 主要工作区        | 做什么                                      |
| ----------------- | ------------------------------------------- |
| `src/app/`        | 路由与页面（`main-window`、`modal-window`） |
| `src/components/` | 业务 UI；`components/ui/` 为 shadcn 基础件  |
| `src/store/`      | Redux 全局态（语言、modal 蒙层等）          |
| `src/hooks/`      | 可复用 React Hooks                          |
| `src/assets/`     | 全局样式 `globals.css`                      |
| `src/lib/`        | 纯工具（如 `cn()`）                         |

| 按需协作                        | 说明                                                    |
| ------------------------------- | ------------------------------------------------------- |
| `src/cmd/`                      | 新增/改 Tauri 调用封装（与 Rust 同步）                  |
| `src/enums/`                    | 新增命令名、事件名、路由等固定字符串                    |
| `src/types/`                    | 事件载荷等 TS 类型                                      |
| `src/generated/`                | **只读**；改 Rust 契约后跑 `bun run generate:contracts` |
| `src/config/`                   | 窗口、i18n 初始化、应用配置                             |
| `src/guards/`、`src/providers/` | 启动、语言、事件总线等壳层                              |
| `src/events/`                   | 跨 Webview → Redux 同步                                 |
| `src/utils/`                    | 前端通用工具（缓存、Tauri event 辅助）                  |

| 通常不直接改                     | 原因                    |
| -------------------------------- | ----------------------- |
| `src-tauri/`                     | 桌面能力与源真相在 Rust |
| `scripts/generate-contracts.mjs` | 除非改代码生成流程      |

---

### Rust / 桌面端工程师（Tauri、系统能力）

| 主要工作区                      | 做什么                                   |
| ------------------------------- | ---------------------------------------- |
| `src-tauri/src/cmd/`            | `#[tauri::command]` 薄入口               |
| `src-tauri/src/context/`        | 跨 Webview 共享 Store（如语言会话）      |
| `src-tauri/src/utils/`          | 无 Store 的业务与通用逻辑                |
| `src-tauri/src/utils/platform/` | Windows / macOS 等平台实现               |
| `src-tauri/src/events/`         | 事件名、emit、listen、handler            |
| `src-tauri/src/contracts.rs`    | `#[typeshare]` 共享 DTO                  |
| `src-tauri/src/lib.rs`          | 注册 command、`.manage`、`events::setup` |

| 按需协作                         | 说明                               |
| -------------------------------- | ---------------------------------- |
| `src-tauri/resources/languages/` | i18n JSON 源文件（与前端展示相关） |
| `src-tauri/tauri.conf.json`      | 应用 id、窗口、打包                |
| `src-tauri/capabilities/`        | Tauri 2 权限能力                   |
| `src-tauri/tauri.*.conf.json`    | 分平台打包配置                     |

| 通常不直接改                  | 原因              |
| ----------------------------- | ----------------- |
| `src/components/`、`src/app/` | UI 属前端         |
| `src/generated/contracts.ts`  | 由 typeshare 生成 |

---

### 全栈 / IPC 负责人（前后端契约）

改一项功能往往**同时**触及：

| 前端                          | Rust                              | 必查文档                               |
| ----------------------------- | --------------------------------- | -------------------------------------- |
| `src/enums/tauri-cmd.ts`      | `src-tauri/src/cmd/*.rs`          | [fullstack-ipc.md](./fullstack-ipc.md) |
| `src/cmd/*.ts`                | `src-tauri/src/lib.rs`（handler） | 同上                                   |
| `src/enums/tauri-event.ts`    | `src-tauri/src/events/names.rs`   | 同上                                   |
| `src/types/tauri-payloads.ts` | `events/payloads.rs`（若有）      | 同上                                   |
| `src/generated/contracts.ts`  | `contracts.rs` + `typeshare.toml` | 跑 `generate:contracts`                |

Modal、新语言、新窗口 label 也有专属清单，见 [fullstack-ipc.md](./fullstack-ipc.md)。

---

### 产品 / 文案（i18n）

| 主要工作区                              | 做什么       |
| --------------------------------------- | ------------ |
| `src-tauri/resources/languages/cn.json` | 简体中文文案 |
| `src-tauri/resources/languages/en.json` | 英文文案     |

| 需开发配合                        | 说明                       |
| --------------------------------- | -------------------------- |
| `src/enums/language.ts`           | 新增语言代码枚举           |
| `src/config/app-config.ts`        | `supportLanguages` 列表    |
| 页面 `useTranslation('命名空间')` | 命名空间需与 JSON 结构一致 |

**不要**把用户可见句子写进 `src/enums/`（enums 只放标识符，不放展示文案）。

---

### UI / 动效

| 主要工作区                              | 做什么                    |
| --------------------------------------- | ------------------------- |
| `src/components/`、`src/components/ui/` | 布局与组件视觉            |
| `src/animation/`                        | GSAP 时间轴、窗口动效封装 |
| `src/assets/globals.css`                | 主题变量、全局样式        |
| `components.json`                       | shadcn 组件来源配置       |

| 注意   |                                                              |
| ------ | ------------------------------------------------------------ |
| 主窗   | 遵守「外层不滚动、内层滚动」见 [frontend.md](./frontend.md)  |
| 无障碍 | 动效需兼容 `prefers-reduced-motion`（`src/animation/core/`） |

---

### 测试 / QA

| 关注                    | 说明                                        |
| ----------------------- | ------------------------------------------- |
| `src/app/main-window/`  | 主窗功能与布局                              |
| `src/app/modal-window/` | 模态窗生命周期                              |
| 多 Webview              | 语言切换是否各窗同步（Rust 会话 + Event）   |
| 打包产物                | `src-tauri/target/`（本地构建，一般不提交） |

自动化测试目录若后续新增，以仓库实际 `**/*.test.*` / `e2e/` 为准；当前模板以手动桌面冒烟为主。

---

### 构建 / DevOps

| 主要工作区                  | 做什么                              |
| --------------------------- | ----------------------------------- |
| `.github/workflows/`        | CI/CD（如 portfolio 注册）          |
| `src-tauri/tauri.conf.json` | 版本号、identifier、bundle          |
| `package.json`              | 前端脚本：`check`、`build`、`tauri` |
| `src-tauri/Cargo.toml`      | Rust 依赖与 crate 配置              |
| `scripts/`                  | 契约生成等维护脚本                  |
| `typeshare.toml`            | Rust → TS 类型生成配置              |

---

## 仓库根目录一览

| 路径              | 职责                  | 谁常改       |
| ----------------- | --------------------- | ------------ |
| `apps/full/`      | 完整控制台 Next 应用  | 前端、动效   |
| `apps/wework/`    | 企微独立 Next 应用    | 前端         |
| `apps/wechat/`    | 微信独立 Next 应用    | 前端         |
| `packages/`       | 前端共享库            | 前端         |
| `src-tauri/`      | Tauri / Rust 桌面端   | Rust、全栈   |
| `docs/`           | 人类文档（含本规范）  | 全员         |
| `scripts/`        | 构建/生成脚本         | DevOps、全栈 |
| `.github/`        | GitHub Actions        | DevOps       |
| `.cursor/rules/`  | Cursor 规则摘要       | 维护者       |
| `.agents/skills/` | Agent 技能与细分 rule | 维护者       |
| `AGENTS.md`       | 给 AI / 新人的总入口  | 维护者       |

---

## `apps/full/src/` 目录树（完整控制台）

```
apps/full/src/
├── app/                 # App Router：页面、layout、error boundary
│   ├── main-window/     # 主窗路由与 Provider
│   └── modal-window/    # 模态 Webview 路由
├── components/          # UI 组件
│   ├── ui/              # shadcn 生成的基础组件
│   ├── title-bar/       # 自定义标题栏（拖拽区等）
│   ├── modal/           # 模态壳 + panels 注册
│   └── error/           # 错误展示
├── cmd/                 # invokeWrapper 封装（对接 TauriCmd）
├── enums/               # 固定字符串（命令、事件、语言、窗口…）
├── types/               # 手写 TS 类型（如事件载荷）
├── generated/           # typeshare 生成（勿手改）
├── store/               # Redux store 与 slices
├── providers/           # Redux、Tauri 事件 Provider
├── guards/              # 启动 / 语言 / 主窗背景等守卫
├── events/              # 跨 Webview 同步到 Redux
├── config/              # 窗口、i18n、app 配置
├── utils/               # 前端通用工具
├── hooks/               # 共享 Hooks
├── animation/           # GSAP 动效模块
├── assets/              # 全局 CSS 等
└── lib/                 # 与 UI 无关的小工具（cn）
```

渠道独立应用见 `apps/wework`、`apps/wechat`；共享 UI 见 `packages/`。

---

## `src-tauri/` 目录树（桌面端）

```
src-tauri/
├── src/
│   ├── main.rs          # 进程入口
│   ├── lib.rs           # Tauri Builder、handler 注册、setup
│   ├── contracts.rs     # #[typeshare] 类型
│   ├── cmd/             # #[tauri::command] 薄层
│   ├── context/         # 跨 Webview Store（.manage）
│   ├── utils/           # 业务与通用逻辑
│   │   └── platform/    # 分 OS 实现（windows / macos）
│   └── events/          # 事件名、emit、listen、handlers
├── resources/
│   └── languages/       # i18n JSON（cn / en）
├── capabilities/        # Tauri 2 权限
├── tauri.conf.json      # 主配置
├── tauri.*.conf.json    # 分平台配置
├── Cargo.toml           # Rust 依赖
└── build.rs             # 构建脚本
```

---

## 协作边界（避免踩坑）

| 场景                     | 正确分工                                              |
| ------------------------ | ----------------------------------------------------- |
| 用户看到的一句话         | 产品/文案改 `resources/languages/*.json`              |
| 命令叫 `get_app_session` | 全栈同时改 Rust cmd + `TauriCmd` + `src/cmd`          |
| 当前语言存在哪           | **源真相** `context/session`；前端 Redux **镜像**     |
| 窗口能否拖动、无边框     | Rust `utils/window` + 前端 `title-bar` + `tauri.conf` |
| 按钮样式                 | 前端 `components/ui`，一般不经过 Rust                 |

---

## 延伸阅读

- 前端写法：[frontend.md](./frontend.md)
- Rust 写法：[backend-rust.md](./backend-rust.md)
- IPC 清单：[fullstack-ipc.md](./fullstack-ipc.md)
- PR 自检：[review-checklist.md](./review-checklist.md)
