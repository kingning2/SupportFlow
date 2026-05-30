# TitleBar Drag Region

## Rule

- 仅 `data-drag-region` 区域触发 `startDragging`。
- 下拉菜单、按钮、输入框必须避免被误识别为拖拽触发源。

## Incorrect

在整个标题栏 `onMouseDown` 直接调用 `startDragging()`。

## Correct

根据事件目标检查：

```tsx
const isDragRegion = Boolean((e.target as HTMLElement).dataset.dragRegion)
if (isDragRegion && e.buttons === 1) {
  void mainWindow.startDragging()
}
```
