# TypeScript 文件夹结构

## 目标

TypeScript 只负责桌面前端界面、状态管理、以及调用 Rust IPC 的薄桥接。

## 核心目录

- `apps/wework/`
  - 个人企业微信桌面前端
- `packages/shared/`
  - 前后端共享类型、IPC 桥接、枚举
- `packages/ui/`
  - 通用 UI 组件与页面壳

## 结构原则

1. 业务页面写在 `apps/*`。
2. 共用 IPC、类型、常量写在 `packages/shared/`。
3. 共用组件写在 `packages/ui/`。
4. TS 不直接承载后端策略，不重复实现 Rust 已有业务规则。
