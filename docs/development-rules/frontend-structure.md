# 前端目录结构规范（仅前端）

本文只约束前端代码（`apps/*`、`packages/*`），不包含 `src-tauri/`。

## 1. 目标

- 目录按职责分层：应用层（`apps`）/ 共享逻辑层（`packages/shared`）/ 共享 UI 层（`packages/ui`）。
- 避免跨层乱引用、魔法字符串分散、同类代码重复出现。
- 新功能默认按本规范落位，不再临时找位置。

## 2. 标准目录树（apps 必须同构）

`apps` 下每个应用目录结构必须一模一样，只允许“平台实现差异”，不允许“目录形态差异”。

```text
apps/<app>/src/
  app/
  components/
  config/
  assets/
  features/
  lib/
```

三端对应关系：

- `apps/full/src` 与渠道 app 至少保持同一层级骨架：`app/`、`components/`、`config/`、`assets/`。
- `apps/full/src` 与渠道 app 至少保持同一层级骨架：`app/`、`components/`、`config/`、`assets/`、`features/`。
- 可以不同：页面实现细节、主题变量、渠道能力、平台分支逻辑。
- 不可以不同：是否存在某一级目录、目录命名、页面路由骨架。

补充约定：

- `apps/full/src` 可以继续保留 `app/main-window/`、`app/modal-window/` 这类桌面控制台专有路由目录，但页面实现同样优先下沉到 `features/full/`。
- 新渠道应用统一使用脚手架骨架：`app/ + assets/ + components/ + config/ + features/<channel>`。
- `apps/full` 同样遵循 `features/full/` 组织私有实现，`app/` 只保留路由入口、布局和错误边界。
- 渠道私有页面和逻辑优先落在 `features/<channel>/`，避免继续在 `src/` 根层散落 `xxx-page.tsx`、`xxx-shell.ts`。

## 3. Monorepo 目录树

```text
apps/
  full/
    src/            # 与其他 app 同构
  wework/
    src/            # 与其他 app 同构
  wechat/
    src/            # 与其他 app 同构

packages/
  shared/
    src/
      tauri-bridge/
        cmd/
        enums/
        window/
      desktop-shell/
        config/
        events/
        guards/
        providers/
        store/
      contracts/
      channel/
      utils/
      index.ts

  ui/
    src/
      app-shell/
      agent-console/
      modal/
      title-bar/
      design-system/
      # 以及 shadcn 基础组件
```

## 4. 各层职责（必须遵守）

- `apps/*`：页面路由、页面级组合、应用私有组件；不沉淀跨应用复用逻辑。
- `packages/shared`：类型、枚举、Tauri 桥接、状态同步、纯函数工具；不放业务展示组件。
- `packages/ui`：可复用 UI 组件、布局壳、设计系统、动效；可调用 shared 暴露的桥接能力，不直接写 IPC 字符串。

## 5. import 边界

- `apps/*` 可以依赖 `@supportflow/shared`、`@supportflow/ui`。
- `packages/ui` 可以依赖 `@supportflow/shared`。
- `packages/shared` 不依赖 `@supportflow/ui`，也不通过 `@supportflow/shared` 反向 import 自己。
- 禁止跨应用直连：`apps/full` 不 import `apps/wework` 或 `apps/wechat` 的源码（反之同理）。

## 6. 文件落位规则

- 新页面：放 `apps/<flavor>/src/app/**/page.tsx`。
- 页面布局：放 `apps/<flavor>/src/app/**/layout.tsx`。
- 页面错误态：放同目录 `error.tsx`。
- 仅当前 app 使用的组件：放 `apps/<flavor>/src/components/`。
- 多 app 复用的组件：放 `packages/ui/src/`。
- IPC 命令封装：放 `packages/shared/src/tauri-bridge/cmd/`。
- 固定字符串（命令名、事件名、路由 key、缓存 key）：放 `packages/shared/src/tauri-bridge/enums/`。
- 展示文案：走 i18n 资源，不放 enums。

## 7. 命名约定

- 目录名：kebab-case。
- 组件文件：kebab-case（例如 `agent-console-app.tsx`）。
- 枚举文件：kebab-case + 语义后缀（例如 `tauri-event.ts`、`local-cache-key.ts`）。
- barrel 导出：每个大目录保留 `index.ts`，只导出对外 API。

## 8. 代码组织建议（前端）

- 一个页面目录最多三类文件：`page.tsx`、`layout.tsx`、`error.tsx`；复杂逻辑下沉到 `components` 或 `packages`。
- 状态优先级：局部 UI 态用 `useState`，跨页面/跨窗口共享态进 `desktop-shell/store`。
- 事件与命令统一走 shared bridge（`invokeWrapper`、`TauriEvent`）。
- 不在页面组件内直接写 `invoke("...")` 或事件名字符串。

## 9. 目录治理清单（PR 自检）

- 新增文件是否放在正确层级（app/shared/ui）？
- 是否引入了跨层反向依赖？
- 是否新增了魔法字符串而未进入 enums？
- 是否把可复用 UI 错放到了 app 私有目录？
- 是否把展示组件错放到了 shared？
- `apps/full`、`apps/wework`、`apps/wechat` 是否仍保持同构目录？

---

如果目录调整涉及 IPC（命令、事件、合同类型），同时参考 `fullstack-ipc.md`。
