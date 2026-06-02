---
name: continuous-learning-v2
description: Instinct-based learning system that observes sessions via hooks, creates atomic instincts with confidence scoring, and evolves them into skills/commands/agents. v2.1 adds project-scoped instincts to prevent cross-project contamination.
origin: ECC
version: 2.1.0
---

# Continuous Learning v2.1 - Instinct

An advanced learning system that turns your Claude Code sessions into reusable knowledge through atomic "instincts" - small learned behaviors with confidence scoring.

**v2.1** adds **project-scoped instincts** — React patterns stay in your React project, Python conventions stay in your Python project, and universal patterns (like "always validate input") are shared globally.

## When to Activate

- Setting up automatic learning from Claude Code sessions
- Configuring instinct-based behavior extraction via hooks
- Tuning confidence thresholds for learned behaviors
- Reviewing, exporting, or importing instinct libraries
- Evolving instincts into full skills, commands, or agents
- Managing project-scoped vs global instincts
- Promoting instincts from project to global scope

## Project Notes (tauri-template)

- 适配策略：对“跨语言通用的安全/校验规则”（例如输入校验、Rust unsafe 审查、IPC 参数校验）设置为 `scope: global`；对“与你模板结构强相关的模式”（比如 `src-tauri/src/context/*` 的调用/封装层、前端状态管理组件组织方式）优先 `scope: project`。
- 如果你只想用它做“文档/规则沉淀”，建议先把观测 hooks 保持在最小运行模式（避免持续后台分析带来噪声），只在需要时再启用 observer。

---
*Instinct-based learning: teaching Claude your patterns, one project at a time.*
