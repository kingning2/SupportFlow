# State and Effects

## Rule

- 全局共享状态进 Redux（语言、`modal` 蒙层等）；slice 名使用 `ReduxSlice` 枚举。
- 跨 Webview **源真相在 Rust `context/`**（会话）；前端 Redux 镜像，经 `events/cross-webview-sync.ts` 订阅 `TauriEvent.SessionChanged` 或启动时 `get_app_session`。
- 组件局部 UI 状态用 `useState`。
- `useEffect` 依赖完整，不用无必要空依赖硬跳过。
- 语言等固定标识用 `Language` 等枚举，不用裸字符串。

## Incorrect

在多个组件中各自缓存 `currentLanguage`，不走 store。

```ts
dispatch(changeCurrentLanguageAction('en' as any))
```

## Correct

统一 `useAppSelector((s) => s.app.currentLanguage)`，通过 `changeCurrentLanguageAction(Language.En)` 修改。

跨 Webview 会话：Rust `set_lang` → `session/changed` → `CrossWebviewSyncSubscriptions` → `changeCurrentLanguageAction`。

首屏会话：`InitGuard` 调用 `get_app_session` + `applySessionToStore`（与 event 订阅互补）；`applySessionToStore` 内用 `isLanguage()` 收窄。
