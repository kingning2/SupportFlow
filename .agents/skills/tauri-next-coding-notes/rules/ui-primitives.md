# UI 基础控件（禁止原生 button / input）

## Rule

在 `apps/**`、`packages/ui/**` 的业务组件里，**不要**直接使用原生 HTML 表单控件作为 UI：

- ❌ `<button>`、`<input>`、`<textarea>`、`<select>`
- ✅ `@supportflow/ui/button` 的 `Button`
- ✅ `@supportflow/ui/input` 的 `Input`
- ✅ `@supportflow/ui/textarea` 的 `Textarea`
- ✅ `@supportflow/ui/select` 的 `Select`

## 允许例外

- `packages/ui/src/**` 内组件库**源码**对原生元素的封装实现。
- 非交互布局容器（`div`、`span` 等）。
- 第三方库要求的底层节点（尽量少改其内部）。

## 示例

```tsx
// ❌
<button type="button" onClick={onSave}>保存</button>
<input value={name} onChange={...} />

// ✅
import { Button } from "@supportflow/ui/button";
import { Input } from "@supportflow/ui/input";

<Button type="button" onClick={onSave}>保存</Button>
<Input value={name} onChange={...} />
```

## Modal footer

Semi `Modal` 的 `footer` 里同样使用 `Button`，不要写原生 `<button>`。
