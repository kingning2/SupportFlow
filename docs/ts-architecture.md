# TypeScript 架构文档

## 定位

TypeScript 是桌面前端层，不是业务主后端。

## 负责内容

- 页面与交互
- 状态管理
- Tauri IPC 调用封装
- 共享类型消费
- 渲染渠道状态

## 分层

- `apps/*`
  - 具体应用页面
- `packages/shared`
  - IPC、共享类型、枚举
- `packages/ui`
  - 通用组件与视图壳

## 与 Rust 的边界

1. TS 不拥有后端策略。
2. TS 通过共享 IPC 封装调用 Rust。
3. TS 只负责把 Rust 返回的状态渲染到界面。
