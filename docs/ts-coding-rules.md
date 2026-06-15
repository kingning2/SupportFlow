# TypeScript 编码规范

1. 优先使用清晰、稳定、可维护的组件与模块边界。
2. 展示文案保持集中管理或在组件内直接使用明确常量，避免同一业务文案在多处散落且互相漂移。
3. 前端尽量保持“状态清晰、桥接薄、业务可读”，不要把复杂后端策略堆到组件里。
4. **业务 UI 统一使用 Semi Design**（`apps/**`、`packages/ui/**`、`packages/shared` 下的 React 组件）：
   - 直接从 `@douyinfe/semi-ui-19` / `@douyinfe/semi-icons` 引入组件（如 `Button`、`Input`、`Modal`、`Select`）。
   - 不要写原生 `<button>`、`<input>`、`<textarea>`、`<select>` 作为可交互控件。
   - `@supportflow/ui/button`、`@supportflow/ui/input` 等路径已 **deprecated**，仅为旧代码兼容；新代码勿再使用。
   - 弹窗使用 Semi `Modal`；应用根布局通过 `DesktopAppLayout` / `SemiProvider` 注入主题。
