# Layout and Scroll

## Rule

- 主窗默认不出现滚动条。
- 父子链路持续传递 `min-h-0`、`flex-1`。
- 主容器优先 `overflow-hidden`，仅在必要子区启用滚动。

## Incorrect

```tsx
<div className="h-full overflow-auto">
  <div className="grid">{children}</div>
</div>
```

## Correct

```tsx
<ContentContainer className="flex flex-col overflow-hidden">
  <div className="flex min-h-0 flex-1 flex-col overflow-hidden">{children}</div>
</ContentContainer>
```
