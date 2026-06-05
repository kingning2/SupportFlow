# SupportFlow Design System

基于 **CSS 变量 + Tailwind v4 `@theme inline`**，默认主题为中性 **zinc**（shadcn new-york）。

## 使用

```css
/* apps/<name>/src/theme.css 或 layout 直接 import */
@import "@supportflow/ui/design-system";
```

渠道应用追加 flavor（仅变量覆盖）：

```css
@import "@supportflow/ui/design-system";
@import "@supportflow/ui/design-system/flavors/wework";
```

```tsx
<body data-flavor="wework">
```

## 自定义

在 app 内新建 `theme.css`，**只写变量**，不要复制 `@theme`：

```css
:root {
  --primary: 240 5.9% 10%;
  --app-bg: #f4f4f5;
}
```

## 目录

| 路径                        | 职责                     |
| --------------------------- | ------------------------ |
| `tokens/semantic.css`       | zinc 默认语义色          |
| `tokens/tailwind-theme.css` | 唯一 `@theme inline`     |
| `tokens/desktop.css`        | 窗体圆角、app-bg         |
| `flavors/*.css`             | `[data-flavor]` 品牌覆盖 |
| `scopes/agent-console.css`  | `.agent-console` 布局    |

## 组件约定

- 使用 `bg-primary`、`text-channel`、`bg-surface-1` 等语义类
- 禁止在 TSX 写品牌 hex；仅 `flavors/` 与 app `theme.css` 可写具体色值
