# 通道 Sidecar（Python 对接 + Rust bridge）

| 层 | 职责 |
|----|------|
| **`channel/`** | 各渠道实现（飞书/微信/企微等）+ `ChannelManager` 启停 |
| **`bridge/`** | 薄封装：`Bridge` → Rust `agent.reply`（无 Python models） |
| **`common/`、`config.py`、`lib/`** | 配置、日志（stderr NDJSON → Rust 落盘）、itchat 等支撑 |
| **Rust** | `AgentRuntime` LLM、`channel_bridge` 配置同步、应用日志 |

## 开发

`tauri dev` 在 **debug** 下优先用源码 `python -m channel`（不会自动装依赖，也**不会**用 `binaries/*.exe`）。

首次开发通道（尤其 **wework**）请在项目根执行：

```bash
bun run bootstrap:sidecar-wheels   # 可选，pip 离线失败时先跑
bun run setup:channel-sidecar-dev  # 用 py -3.8 安装 requests / ntwork / pilk 等
```

脚本会提示把 `COW_PYTHON_EXECUTABLE=...` 写入根目录 `.env`，然后**重新** `bun run tauri dev`（`build.rs` 会把 `.env` 编进二进制；未配置时 Windows 会自动尝试 `py -3.8`）。

手动调试：

```bash
cd src-tauri/cowagent
set COW_CONFIG_PATH=..\resources\config.json
py -3.8 -m channel
```

Tauri 启动时会注入 `COW_CONFIG_PATH` / `COW_TAURI_MODE=1`。

**wework** 要求 **Python 3.8–3.10**（3.12 无 ntwork wheel）。`ntwork` 不在 PyPI，由 `setup:channel-sidecar-dev` / 打包脚本从 [ntwork-bin-backup](https://github.com/hanfangyuan4396/ntwork-bin-backup) 安装。

另需：Windows、企业微信 PC 版（建议 4.0.8.6027）、客户端已登录。

## 构建 exe

```bash
# 1) 用可用的 pip（通常 python 3.12）预下载 PyInstaller、requests、渠道依赖、pilk 等 wheel
bun run bootstrap:sidecar-wheels

# 2) 打包（自动下载 ntwork wheel + 安装 requirements-sidecar.txt + PyInstaller）
bun run build:channel-sidecar
```

`requirements-sidecar.txt` 列出打进 exe 的第三方库（`requests`、`wechatpy`、飞书/钉钉等）。未安装就打包容易出现 `No module named 'requests'`。

环境变量：

| 变量 | 说明 |
|------|------|
| `COW_SIDECAR_PYTHON` | 打包用解释器（建议 3.8） |
| `COW_BOOTSTRAP_PYTHON` | 下载 wheel 用的解释器（默认 `python`，常为 3.12） |
| `COW_NTWORK_WHEEL` | ntwork `.whl` 的绝对路径 |
| `COW_SKIP_WEWORK_DEPS` | `1` 时跳过 wework 依赖检查 |

产物：`src-tauri/binaries/cowagent-channels-<target>.exe`（PyInstaller **单文件**，内含 Python 3.8 运行时 + ntwork/pilk 等，不依赖用户机上的 Python）。

Release 构建时该 exe 会以 **`include_bytes` 嵌入 `tauri-app.exe`**，首次运行解压到应用缓存目录，安装包旁不再单独带 `cowagent-channels-*.exe`。

若打包提示 **file is in use**，先退出应用，再执行 `bun run build:channel-sidecar` 或 `bun run finalize:channel-sidecar`。

注意：exe 里带上 ntwork 只解决 **Python 依赖**；用户机器仍须自行安装并登录 **企业微信 PC 客户端**。
