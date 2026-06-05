# Rust 主架构对接 Python Sidecar 设计稿

## 定位

这份文档只回答一个问题：Rust 这一侧应该如何设计，才能以可维护、可演进、全异步、非阻塞 UI 的方式对接 Python sidecar。

这里的前提固定如下：

- Rust 是桌面端唯一应用后端
- Python 是独立 sidecar 进程，只负责渠道 SDK 执行
- 前端只面对 Rust，不面对 Python
- Rust 与 Python 必须按多进程通信架构设计，而不是同进程思维设计
- 任何 sidecar 对接都不能阻塞 Tauri UI

这份文档是 `docs/rust-architecture.md` 与 `docs/python-architecture.md` 的实施补充，不重复通用编码规范与目录规范，只定义 Rust 如何主导 sidecar 协作。

## 核心原则

### 1. Rust 是唯一应用编排层

Rust 负责：

- Tauri command 和 event
- 渠道配置持久化
- sidecar 生命周期管理
- 渠道运行态快照
- 登录态映射
- 首次同步判定
- 重试与重启策略
- 前端状态广播
- AI、知识库、工具链等核心业务能力

Python 只负责：

- `wechat` / `wework` SDK 登录
- 消息接收与发送
- 媒体下载
- 必要的 SDK 适配
- 向 Rust 回调核心能力
- 向 Rust 上报运行状态

如果某段逻辑需要被前端理解、需要持久化、需要跨 Webview 共享、需要做策略判断，那么它必须放在 Rust，而不是 Python。

### 2. 异步优先，绝不阻塞 UI

Rust 对 sidecar 的所有交互默认都应使用 `async/await`。

尤其是以下流程，必须设计为异步后台流程：

- sidecar 启动
- channel 启动、停止、重启
- 联系人同步
- 需要等待秒级以上的 SDK 初始化
- 可能卡住 I/O 的调用
- 可能失败并需要重试的流程

规则固定如下：

- `cmd/` 只受理动作，不长时间等待最终完成
- 长任务必须由 Rust 后台异步执行
- 前端刷新依赖 Rust event 和 Rust 状态快照，不依赖阻塞式 command 返回
- 不允许在 command 内 `sleep` 等待 Python ready
- 不允许用“等 sidecar 最终返回成功”作为 UI 解锁条件

### 3. 多进程隔离

Python sidecar 是独立进程，不是 Rust 的扩展模块。

这意味着：

- Rust 不依赖 Python 内部内存态
- Rust 不依赖 Python 内部线程模型
- Rust 不依赖 Python 内部对象引用
- Rust 只依赖清晰、窄化的进程间协议

Rust 必须始终把 Python 看成一个可以重启、可以超时、可以替换、可以断开的外部进程。

## Rust 侧推荐分层

### `cmd/`

`cmd/` 只做 Tauri command 入口。

职责：

- 参数校验
- 权限边界
- 返回值类型定义
- 一行委托给 `context`

禁止项：

- 不在 `cmd/` 里直接拼 sidecar RPC
- 不在 `cmd/` 里直接操作 `stdin/stdout`
- 不在 `cmd/` 里等待长耗时 sidecar 流程完成
- 不在 `cmd/` 里写重试、重连、同步判定等业务策略

对启动渠道、联系人同步、重连等动作，`cmd/` 只能：

- 立刻返回“已受理”
- 或返回当前 Rust 快照

真正执行必须放到 `context` 的异步协调层。

### `context/`

`context/` 是 Rust 管理 Python sidecar 的核心层。所有 Rust 与 Python 的通信，都只允许收口在这里。

#### `channel_python_sidecar`

它是异步多进程 RPC 客户端层，负责：

- 持有 sidecar 进程句柄
- 持有 `stdin` / `stdout` / `stderr`
- 启动 sidecar
- 存活检查
- request id -> pending future 映射
- stdout 异步 reader loop
- stderr 日志转发
- 请求超时
- 断线清理
- Rust typed method 到 Python RPC method 的映射

推荐约束：

- 上层不能直接传裸 RPC 字符串到处调用
- 对外只暴露领域化方法，例如：
  - `channel_start(channel)`
  - `channel_stop(channel)`
  - `channel_restart(channel)`
  - `wework_sync_contacts()`

这样可以把 Python 协议细节压缩在单一文件中。

#### `channel_runtime`

它是动作编排与后台任务层，负责：

- 渠道配置写入
- 渠道连接/断开/重启编排
- 配置变更与运行时动作拆分
- 是否需要重启的判定
- 长任务后台异步启动
- 调用 sidecar 前后的 Rust 状态协调

推荐规则：

- 单次 RPC 超时放在 `channel_python_sidecar`
- 是否重试、何时重试、何时回退放在 `channel_runtime`
- 不要让前端或 `cmd/` 知道 sidecar 重试细节

#### `channel_status`

它是 Rust 本地状态真相层，负责：

- 保存运行态快照
- 接收 Python 推送的状态变化
- 把 Python phase 归一为 Rust 领域状态
- 提供前端只读快照

前端读取的一切渠道状态，都必须来自 Rust store，而不是 Python 原始 phase。

#### `channel_console_api`

它是前端领域接口翻译层，负责：

- 聚合前端可见控制台动作
- 把前端请求翻译为 Rust 领域行为
- 把 Rust 状态翻译为前端响应结构

禁止项：

- 不直接暴露 Python RPC method 名称
- 不直接把 sidecar 协议返回给前端

### `events/`

`events/` 只负责 Rust -> Frontend 的广播契约。

规则固定如下：

- Python 不直接给前端发事件
- 所有 Python 事件先进入 Rust
- Rust 更新本地 store
- Rust 再广播前端

前端永远只订阅 Rust 定义的 event。

### `crates/*`

真正的核心能力放在 Rust crates 中，例如：

- AI 对话
- 知识库
- 媒体处理
- 消息加工
- 富文本处理

Python 收到渠道消息后，只允许回调 Rust 请求这些能力，不能在 sidecar 内重新长出 AI 编排层。

## 多进程通信模型

### 进程模型

- Rust 主进程：应用编排与状态中心
- Python sidecar：独立子进程
- 通信方式：双向异步 NDJSON RPC over stdio

这里的关系不是两个平级后端，而是：

- Rust 是宿主
- Rust 是连接管理者
- Rust 是状态聚合者
- Python 是受控执行器

### 允许的消息分类

只允许三类消息。

#### 1. Rust -> Python 命令 RPC

典型动作：

- 启动渠道
- 停止渠道
- 重启渠道
- 手动联系人同步

特点：

- 短指令
- 带唯一 request id
- 有明确 timeout
- 返回原子执行结果
- 不负责前端最终态解释

#### 2. Python -> Rust 能力请求 RPC

典型动作：

- `agent.reply`
- `channel.process`
- `channel.decorate_text`
- `channel.extract_media`

特点：

- Python 只在需要 Rust 核心能力时发起
- Rust 负责业务执行
- Python 不继续叠加策略层编排

#### 3. Python -> Rust 状态通知

典型动作：

- 登录阶段变化
- 二维码变化
- 联系人同步完成
- 渠道进程状态变化

特点：

- 单向通知
- Rust 负责更新 store
- Rust 负责持久化
- Rust 负责广播前端

### 通信约束

所有 Rust <-> Python IPC 必须满足：

- 所有 RPC 都有 timeout
- 所有请求都有唯一 id
- sidecar 断线时必须清理所有 pending response
- stdout 必须由异步 loop 持续读取
- 禁止同步阻塞读取 stdout
- stderr 只做日志转发，不参与业务协议
- 前端可见错误文案由 Rust 归一化

## 异步与非阻塞设计模板

### 模板一：立即返回 + 后台执行

适用于：

- 首次联系人同步
- 渠道启动
- 渠道重启
- 长耗时 SDK 初始化

流程：

1. 前端发 command
2. Rust command 立即返回“已受理”或当前状态
3. Rust 在后台 `tokio::spawn` 执行 sidecar RPC
4. Python 执行过程中不断通知状态变化
5. Rust 写入 store 并广播 event
6. 前端靠订阅状态刷新 UI

这个模板下，前端不应该死等 command 最终返回“完成”。

### 模板二：状态查询与动作触发分离

适用于：

- 登录态
- 二维码态
- 同步态

规则：

- `GET` 类接口只读 Rust 快照
- `POST` 类接口只触发动作
- `POST` 不承诺同步等待最终结果
- UI 展示依赖 event + 状态快照

这可以避免把一个本该异步推进的流程错误设计成同步请求。

### 模板三：超时与重试分层

分层规则固定如下：

- sidecar 单次 RPC timeout 放在 `channel_python_sidecar`
- 重试策略放在 `channel_runtime`
- 前端只消费当前状态、失败信息、是否允许重试

不要把超时、重试、重启分散在前端、`cmd/` 和 sidecar 三处分别维护。

## 维护性规则

新增任何渠道相关需求时，先按下面五条判断。

### 1. 是否依赖 SDK 句柄、渠道线程、Python 第三方库对象

- 是：留 Python
- 否：放 Rust

### 2. 是否需要前端理解、展示或跨 Webview 共享

- 是：状态真相必须放 Rust

### 3. 是否属于策略判断

- 是：必须放 Rust

典型例子：

- 是否视为已登录
- 是否首次同步
- 是否后台补同步
- 是否需要自动重启
- 是否允许前端按钮点亮

### 4. 是否可能耗时或阻塞

- 是：必须后台异步执行

禁止把这种逻辑直接挂在 UI 同步调用链上。

### 5. 是否把 Python 内部协议暴露给前端

- 是：说明设计错误，必须收回 Rust 做翻译层

前端只能依赖 Rust 领域接口和 Rust 领域状态，不能感知 Python 内部 method、phase、数据结构细节。

## 标准调用流

### 渠道登录流

1. 前端触发登录
2. Rust 受理动作并确保 sidecar 运行
3. Rust 异步调用 Python 启动渠道
4. Python 推送二维码、扫描、登录状态
5. Rust 把 Python phase 映射为前端可见状态
6. Rust 广播 event
7. 前端只订阅 Rust 状态

关键点：

- 登录态解释在 Rust
- Python 只上报事实
- UI 不等待 sidecar 同步完成

### 联系人同步流

1. Rust 判断是否首次登录或是否用户手动触发
2. 如果需要同步，Rust 后台启动同步任务
3. Python 执行联系人同步
4. Python 完成后通知 Rust
5. Rust 写入“已同步”持久化状态
6. 前端只看到状态推进，不等待阻塞请求

关键点：

- 是否需要同步由 Rust 决定
- 同步完成标记由 Rust 持久化
- Python 不持有“是否同步过”的业务真相

### 消息处理流

1. Python 收到渠道消息
2. Python 做最小消息解析
3. Python 回调 Rust 请求核心能力
4. Rust 异步执行 AI、知识库、工具链
5. Rust 返回回复结果
6. Python 发回渠道
7. 若失败，Rust 返回结构化错误，Python 只回传失败事实

关键点：

- AI 与业务编排在 Rust
- Python 只负责渠道接入
- 核心回复链不在 Python 内继续生长

## 与现有代码的对照锚点

这份文档只引用以下关键锚点：

- `src-tauri/src/context/channel_python_sidecar.rs`
  - 异步多进程 RPC 客户端层
- `src-tauri/src/context/channel_runtime.rs`
  - 动作编排与后台任务层
- `src-tauri/src/context/channel_status.rs`
  - Rust 本地状态真相层
- `src-tauri/src/context/channel_console_api.rs`
  - 前端领域接口翻译层

## 未来演进方向

后续继续重构时，按下面顺序推进：

1. 继续把 Python 中的策略判断迁回 Rust
2. 继续把 Python RPC 名字封装到 Rust typed method 中
3. 继续让前端只依赖 Rust 领域状态
4. 继续压平 Python 中非 SDK 必需的通用壳层
5. 禁止在 Python 中新增应用级 AI 或配置编排模块

目标形态不是“双核心架构”，而是：

- Rust 稳定
- Python 更薄
- 前端更单纯
- 协议更窄
- 故障边界更清晰
