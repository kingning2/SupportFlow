# Examples

## 1) 新增 Tauri 命令（最小闭环）

```rust
// src-tauri/src/cmd/system.rs
#[tauri::command]
pub async fn get_system_mode() -> Result<String, String> {
    Ok("normal".to_string())
}
```

```rust
// src-tauri/src/cmd/mod.rs
pub mod system;
```

```rust
// src-tauri/src/lib.rs
.invoke_handler(tauri::generate_handler![
  // ...
  cmd::system::get_system_mode,
])
```

```ts
// packages/shared/src/tauri-bridge/enums/tauri-cmd.ts
export enum TauriCmd {
  // ...
  GetSystemMode = "get_system_mode"
}
```

```ts
// packages/shared/src/tauri-bridge/cmd/system.ts
import { TauriCmd } from "../enums/tauri-cmd";
import { invokeWrapper } from "./invoke";

export const getSystemMode = () => invokeWrapper<string>(TauriCmd.GetSystemMode);
```

## 2) i18n 文案新增

```tsx
const { t } = useTranslation("title_bar");
<Button>{t("menu_about")}</Button>;
```

```json
// src-tauri/resources/languages/cn.json
{
  "title_bar": {
    "menu_about": "關於"
  }
}
```

```json
// src-tauri/resources/languages/en.json
{
  "title_bar": {
    "menu_about": "About"
  }
}
```

## 3) 无滚动条主窗

```tsx
<ContentContainer className="flex flex-col overflow-hidden">
  <div className="flex h-full min-h-0 flex-1 flex-col gap-3 overflow-hidden p-3">
    <div className="min-h-0 flex-1 overflow-hidden">{/* 内容 */}</div>
  </div>
</ContentContainer>
```

## 4) 错误边界与日志

```tsx
"use client";
import { useEffect } from "react";
import { log } from "@supportflow/shared/tauri-bridge/cmd/log";
import { FeLogLevel } from "@supportflow/shared/tauri-bridge/enums";

export default function Error({ error, reset }: { error: Error; reset: () => void }) {
  useEffect(() => {
    void log(FeLogLevel.Error, `main-window error: ${error.message}`);
  }, [error]);
  return <button onClick={reset}>Try again</button>;
}
```

## 5) 跨 Webview 状态同步

```tsx
// 根布局已挂载 TauriEventProvider → CrossWebviewSyncSubscriptions
// 会话 Redux 无需页面手写订阅

import { applySessionToStore } from "@supportflow/shared/desktop-shell/events/cross-webview-sync";
import { getAppSession } from "@supportflow/shared/tauri-bridge/cmd/session";

void getAppSession().then((session) => applySessionToStore(store, session));
```

```tsx
// 自定义事件监听（须在 TauriEventProvider 子树内）
import { TauriEvent } from "@supportflow/shared/tauri-bridge/enums";
import { useTauriEventApi } from "@supportflow/shared/desktop-shell/providers/tauri-event-provider";

const { on } = useTauriEventApi();
useEffect(() => on(TauriEvent.ModalOpened, handler), [on]);
```

```ts
// 前端 → Rust 日志（非 invoke）
import { log } from "@supportflow/shared/tauri-bridge/cmd/log";
import { FeLogLevel } from "@supportflow/shared/tauri-bridge/enums";

void log(FeLogLevel.Info, "modal opened");
```

## 6) 新增 Modal 面板

```ts
// packages/shared/src/tauri-bridge/enums/modal-panel.ts
export enum ModalPanel {
  Demo = "demo",
  Settings = "settings"
}
```

```tsx
// apps/full/src/components/modal/panels/settings.tsx — 实现 SettingsPanel
```

```ts
// apps/full/src/components/modal/panels/index.ts
[ModalPanel.Settings]: SettingsPanel
```

```tsx
import { ModalPanel } from "@supportflow/shared/tauri-bridge/enums";
import { useModalWindow } from "@supportflow/ui/modal";

const { openModal } = useModalWindow();
await openModal({ name: ModalPanel.Settings, width: 520, height: 400 });
```
