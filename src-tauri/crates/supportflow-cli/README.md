# supportflow-cli

Rust port of SupportFlow Agent `cli/` — the **`sf`** command-line tool for SupportFlow.

## Build & run

```bash
# from repo root
cargo build --manifest-path src-tauri/Cargo.toml -p supportflow-cli --release
cargo run --manifest-path src-tauri/Cargo.toml -p supportflow-cli -- --help

# or
pnpm run sf -- --help
```

Install the binary:

```bash
cargo install --path src-tauri/crates/supportflow-cli --locked
```

## Environment

| Variable | Purpose |
|----------|---------|
| `SUPPORT_FLOW_WORKSPACE` | Agent workspace (skills, memory, knowledge). Default: `%APPDATA%/SupportFlow` (Windows) or `~/.local/share/SupportFlow` |
| `CHANNEL_CONFIG_PATH` | `config.json` path (models/API keys). Default: workspace `config.json` or dev `src-tauri/resources/config.json` |
| `SUPPORTFLOW_APP` | Path to desktop `tauri-app` exe for `sf start` |

## Commands

| Command | Description |
|---------|-------------|
| `sf version` / `sf help` | Version and usage |
| `sf start` / `stop` / `restart` / `status` / `logs` | Desktop app process (PID file in workspace) |
| `sf agent chat` / `sf agent repl` | Headless agent via Rust `bridge` |
| `sf skill list` / `search` / `install` / `enable` / `disable` / `info` | Skills + Skill Hub |
| `sf knowledge` / `list` / `upload` | Knowledge base files |
| `sf config show` | Paths and model summary |
| `sf context clear` | Clear persisted session in SQLite |
| `sf install-browser` | Browser tool setup hints (Chrome/CDP) |

Memory index management remains in chat: `/memory status`, `/memory rebuild-index`.
