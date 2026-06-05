# Frontend Monorepo Packages

**只放公用代码**，渠道私有逻辑在 `apps/<channel>/`。

```
packages/
  shared/     工具、类型、contracts、Tauri IPC、Redux、渠道表单逻辑
  ui/         shadcn、标题栏、Modal、控制台、渠道壳布局
```

## 依赖方向

```
shared → ui → apps/full
shared + ui → apps/wework | apps/wechat（渠道页在各自 app 内）
```

## 导入示例

```ts
import { cn, fetchChannels } from "@supportflow/shared";
import { invokeWrapper } from "@supportflow/shared/tauri-bridge/cmd/invoke";
import StoreProvider from "@supportflow/shared/desktop-shell/providers/store";
import { AgentConsoleApp } from "@supportflow/ui/agent-console";
import { DesktopAppLayout, AppShellLayout } from "@supportflow/ui/app-shell";
```

渠道根布局（`apps/wework/src/app/layout.tsx`）：

```tsx
import { DesktopAppLayout } from "@supportflow/ui/app-shell";
import "@supportflow/ui/design-system";
import { weworkShellAccent } from "@/shell-accent";

<DesktopAppLayout accent={weworkShellAccent}>{children}</DesktopAppLayout>;
```

渠道页示例（在 `apps/wework/src/`）：

```ts
import { WeworkPage } from "@/wework-page";
```

## Turborepo

```bash
pnpm run typecheck
pnpm run generate:contracts   # → packages/shared/src/contracts/contracts.ts
```
