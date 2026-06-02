# Preflight Checks

## 必做检查

1. 前端：`pnpm run check`（含 `prettier --check`、`eslint`、`turbo run typecheck`）
2. Rust 改动时：`pnpm run check:rust`（`cargo check`）
3. 改了 `#[typeshare]` 契约：`pnpm run generate:contracts`（CI 可用 `pnpm run check:contracts` 校验 `packages/shared/src/contracts/contracts.ts`）
4. 新增 `TauriCmd` / `TauriEvent` / `ModalPanel` 等：双端字符串与 Rust `names.rs` / `lib.rs` handler 一致
5. i18n key 双语同步（`cn.json` / `en.json`）
6. 标题栏拖拽与菜单交互互不冲突
7. 主窗常见尺寸下无意外滚动条
8. 前端业务代码中无新增魔法字符串（应进 `packages/shared/src/tauri-bridge/enums/`）

## 建议

- 提交前手测：语言切换、示例 Modal 打开/关闭、窗口最小化/关闭。
- 多 flavor 构建：`pnpm run build:flavor:full` / `wework` / `wechat`（见根 `package.json`）。
