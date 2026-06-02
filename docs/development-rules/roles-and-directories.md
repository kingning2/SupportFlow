# 角色与目录地图

新人或协作时，先确认**自己的职责**，再对照下表找「主要改哪里 / 少碰哪里」。  
技术调用链见 [architecture.md](../../.agents/skills/tauri-next-coding-notes/architecture.md)。

---

## 按角色：你该动哪些目录

### 前端工程师（页面 / 组件 / 状态）

| 主要工作区                             | 做什么                                          |
| -------------------------------------- | ----------------------------------------------- |
| `apps/full/src/app/`                   | 完整控制台路由（`main-window`、`modal-window`） |
| `apps/full/src/components/`            | 应用私有 UI（如 modal panels）                  |
| `apps/wework/src/`、`apps/wechat/src/` | 渠道页与 accent                                 |
| `packages/ui/src/`                     | 控制台、shadcn、标题栏、Modal                   |
| `packages/shared/src/desktop-shell/`   | Redux、Provider、守卫、跨 Webview 同步          |

| 按需协作                                    | 说明                                                 |
| ------------------------------------------- | ---------------------------------------------------- |
| `packages/shared/src/tauri-bridge/cmd/`     | 新增/改 Tauri 调用封装（与 Rust 同步）               |
| `packages/shared/src/tauri-bridge/enums/`   | 命令名、事件名、路由等固定字符串                     |
| `packages/shared/src/contracts/`            | 事件载荷与 typeshare 生成物（**contracts.ts 只读**） |
| `apps/full/src/config/`                     | 窗口行为配置                                         |
| `packages/shared/src/desktop-shell/config/` | i18n 初始化、应用配置                                |

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

| 通常不直接改                                 | 原因              |
| -------------------------------------------- | ----------------- |
| `packages/ui/`、`apps/*/src/app/`            | UI 属前端         |
| `packages/shared/src/contracts/contracts.ts` | 由 typeshare 生成 |

---

### 全栈 / IPC 负责人（前后端契约）

改一项功能往往**同时**触及：

| 前端                                              | Rust                              | 必查文档                               |
| ------------------------------------------------- | --------------------------------- | -------------------------------------- |
| `packages/shared/.../enums/tauri-cmd.ts`          | `src-tauri/src/cmd/*.rs`          | [fullstack-ipc.md](./fullstack-ipc.md) |
| `packages/shared/.../tauri-bridge/cmd/*.ts`       | `src-tauri/src/lib.rs`（handler） | 同上                                   |
| `packages/shared/.../enums/tauri-event.ts`        | `src-tauri/src/events/names.rs`   | 同上                                   |
| `packages/shared/.../contracts/tauri-payloads.ts` | `events/payloads.rs`（若有）      | 同上                                   |
| `packages/shared/.../contracts/contracts.ts`      | `contracts.rs` + `typeshare.toml` | 跑 `generate:contracts`                |

Modal、新语言、新窗口 label 也有专属清单，见 [fullstack-ipc.md](./fullstack-ipc.md)。

---

### 产品 / 文案（i18n）

| 主要工作区                              | 做什么       |
| --------------------------------------- | ------------ |
| `src-tauri/resources/languages/cn.json` | 简体中文文案 |
| `src-tauri/resources/languages/en.json` | 英文文案     |

| 需开发配合                                               | 说明                       |
| -------------------------------------------------------- | -------------------------- |
| `packages/shared/.../enums/language.ts`                  | 新增语言代码枚举           |
| `packages/shared/.../desktop-shell/config/app-config.ts` | `supportLanguages` 列表    |
| 页面 `useTranslation('命名空间')`                        | 命名空间需与 JSON 结构一致 |

**不要**把用户可见句子写进 enums（enums 只放标识符，不放展示文案）。

---

### UI / 动效

| 主要工作区                         | 做什么                          |
| ---------------------------------- | ------------------------------- |
| `packages/ui/src/`                 | 控制台、shadcn、标题栏、Modal   |
| `packages/ui/src/modal/motion/`    | GSAP 窗口动效                   |
| `apps/full/src/assets/globals.css` | 完整控制台全局样式              |
| `apps/full/components.json`        | shadcn 组件来源配置（full app） |

| 注意   |                                                             |
| ------ | ----------------------------------------------------------- |
| 主窗   | 遵守「外层不滚动、内层滚动」见 [frontend.md](./frontend.md) |
| 无障碍 | 动效需兼容 `prefers-reduced-motion`                         |

---

### 测试 / QA

| 关注                              | 说明                                        |
| --------------------------------- | ------------------------------------------- |
| `apps/full/src/app/main-window/`  | 主窗功能与布局                              |
| `apps/full/src/app/modal-window/` | 模态窗生命周期                              |
| 多 Webview                        | 语言切换是否各窗同步（Rust 会话 + Event）   |
| 打包产物                          | `src-tauri/target/`（本地构建，一般不提交） |

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

## `apps/full/src/` 目录树（完整控制台，应用层）

```
apps/full/src/
├── app/                 # App Router：页面、layout、error boundary
│   ├── main-window/     # 主窗路由与 Provider
│   └── modal-window/    # 模态 Webview 路由
├── components/          # 应用私有组件
│   ├── modal/panels/    # Modal 面板实现与注册
│   └── error/           # 错误展示
├── config/              # 窗口配置（re-export shared/ui）
├── guards/              # 主窗背景等应用级守卫
├── assets/              # 全局 CSS
└── lib/                 # 应用内小工具（如 cn 别名）
```

IPC、Redux、控制台 UI 在 `packages/shared`、`packages/ui`。渠道应用见 `apps/wework`、`apps/wechat`。

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
| 命令叫 `get_app_session` | 全栈同时改 Rust cmd + `TauriCmd` + `tauri-bridge/cmd` |
| 当前语言存在哪           | **源真相** `context/session`；前端 Redux **镜像**     |
| 窗口能否拖动、无边框     | Rust `utils/window` + 前端 `title-bar` + `tauri.conf` |
| 按钮样式                 | 前端 `components/ui`，一般不经过 Rust                 |

---

## 延伸阅读

- 前端写法：[frontend.md](./frontend.md)
- Rust 写法：[backend-rust.md](./backend-rust.md)
- IPC 清单：[fullstack-ipc.md](./fullstack-ipc.md)
- PR 自检：[review-checklist.md](./review-checklist.md)
