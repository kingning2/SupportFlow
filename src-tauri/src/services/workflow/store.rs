//! Workflow 运行态 SQLite 持久化（独立于 `memory/long-term/index.db`）。

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use rusqlite::{params, Connection};
use serde_json::Value;
use tracing::debug;

use super::types::{NodeKind, RunStatus, StepRecord, StepStatus, WorkflowContext, WorkflowRun};

const DDL: &str = r"
CREATE TABLE IF NOT EXISTS workflow_runs (
    id               TEXT PRIMARY KEY,
    definition_id    TEXT    NOT NULL,
    status           TEXT    NOT NULL,
    current_node_id  TEXT,
    context_json     TEXT    NOT NULL DEFAULT '{}',
    session_id       TEXT,
    error            TEXT,
    created_at       TEXT    NOT NULL,
    updated_at       TEXT    NOT NULL
);
CREATE TABLE IF NOT EXISTS workflow_steps (
    id           TEXT PRIMARY KEY,
    run_id       TEXT    NOT NULL,
    node_id      TEXT    NOT NULL,
    node_kind    TEXT    NOT NULL,
    status       TEXT    NOT NULL,
    started_at   TEXT    NOT NULL,
    finished_at  TEXT,
    input_json   TEXT,
    output_json  TEXT,
    error        TEXT,
    seq          INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (run_id) REFERENCES workflow_runs(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_workflow_steps_run ON workflow_steps (run_id, seq);
CREATE TABLE IF NOT EXISTS workflow_events (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id       TEXT    NOT NULL,
    step_id      TEXT,
    event_type   TEXT    NOT NULL,
    payload_json TEXT    NOT NULL DEFAULT '{}',
    created_at   INTEGER NOT NULL,
    FOREIGN KEY (run_id) REFERENCES workflow_runs(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_workflow_events_run ON workflow_events (run_id, id);
";

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn run_status_to_str(status: RunStatus) -> &'static str {
    match status {
        RunStatus::Pending => "pending",
        RunStatus::Running => "running",
        RunStatus::WaitingHuman => "waiting_human",
        RunStatus::Paused => "paused",
        RunStatus::Succeeded => "succeeded",
        RunStatus::Failed => "failed",
        RunStatus::Cancelled => "cancelled",
    }
}

fn run_status_from_str(raw: &str) -> Option<RunStatus> {
    match raw {
        "pending" => Some(RunStatus::Pending),
        "running" => Some(RunStatus::Running),
        "waiting_human" => Some(RunStatus::WaitingHuman),
        "paused" => Some(RunStatus::Paused),
        "succeeded" => Some(RunStatus::Succeeded),
        "failed" => Some(RunStatus::Failed),
        "cancelled" => Some(RunStatus::Cancelled),
        _ => None,
    }
}

fn step_status_to_str(status: StepStatus) -> &'static str {
    match status {
        StepStatus::Queued => "queued",
        StepStatus::Active => "active",
        StepStatus::Suspended => "suspended",
        StepStatus::Completed => "completed",
        StepStatus::Failed => "failed",
    }
}

fn step_status_from_str(raw: &str) -> Option<StepStatus> {
    match raw {
        "queued" => Some(StepStatus::Queued),
        "active" => Some(StepStatus::Active),
        "suspended" => Some(StepStatus::Suspended),
        "completed" => Some(StepStatus::Completed),
        "failed" => Some(StepStatus::Failed),
        _ => None,
    }
}

fn node_kind_to_str(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::AgentReply => "agent_reply",
        NodeKind::ToolCall => "tool_call",
        NodeKind::HumanAndsign => "human_andsign",
        NodeKind::Branch => "branch",
        NodeKind::Delay => "delay",
    }
}

fn node_kind_from_str(raw: &str) -> Option<NodeKind> {
    match raw {
        "agent_reply" => Some(NodeKind::AgentReply),
        "tool_call" => Some(NodeKind::ToolCall),
        "human_andsign" => Some(NodeKind::HumanAndsign),
        "branch" => Some(NodeKind::Branch),
        "delay" => Some(NodeKind::Delay),
        _ => None,
    }
}

/// 返回工作区内的 workflow 数据库路径：`{workspace}/workflow/runs.db`。
pub fn db_path_for_workspace(workspace: &Path) -> PathBuf {
    workspace.join("workflow").join("runs.db")
}

/// Workflow 运行态存储（独立 SQLite，与会话/记忆库分离）。
pub struct WorkflowStore {
    conn: Mutex<Connection>,
}

impl WorkflowStore {
    /// 打开或创建指定路径的 workflow 数据库。
    pub fn open(db_path: &Path) -> Result<Self, String> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| e.to_string())?;
        conn.execute_batch(DDL).map_err(|e| e.to_string())?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")
            .map_err(|e| e.to_string())?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// 按工作区路径打开 workflow 存储。
    pub fn for_workspace(workspace: &Path) -> Result<Arc<Self>, String> {
        let db_path = db_path_for_workspace(workspace);
        Ok(Arc::new(Self::open(&db_path)?))
    }

    /// 持久化新的 workflow run（含初始 steps）。
    pub fn create_run(&self, run: &WorkflowRun) -> Result<(), String> {
        let context_json = serde_json::to_string(&run.context).map_err(|e| e.to_string())?;
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

        tx.execute(
            "INSERT INTO workflow_runs
                (id, definition_id, status, current_node_id, context_json,
                 session_id, error, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                run.id,
                run.definition_id,
                run_status_to_str(run.status),
                run.current_node_id,
                context_json,
                run.session_id,
                run.error,
                run.created_at,
                run.updated_at,
            ],
        )
        .map_err(|e| e.to_string())?;

        for (seq, step) in run.steps.iter().enumerate() {
            Self::insert_step_tx(&tx, &run.id, step, seq as i64)?;
        }

        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 更新 run 元数据（status、current_node、context 等）。
    pub fn update_run(&self, run: &WorkflowRun) -> Result<(), String> {
        let context_json = serde_json::to_string(&run.context).map_err(|e| e.to_string())?;
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE workflow_runs SET
                status = ?2,
                current_node_id = ?3,
                context_json = ?4,
                session_id = ?5,
                error = ?6,
                updated_at = ?7
             WHERE id = ?1",
            params![
                run.id,
                run_status_to_str(run.status),
                run.current_node_id,
                context_json,
                run.session_id,
                run.error,
                run.updated_at,
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 插入或更新单步记录。
    pub fn update_step(&self, run_id: &str, step: &StepRecord, seq: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

        let exists: bool = tx
            .query_row(
                "SELECT 1 FROM workflow_steps WHERE id = ?1",
                params![step.id],
                |_| Ok(()),
            )
            .is_ok();

        if exists {
            Self::update_step_tx(&tx, step)?;
        } else {
            Self::insert_step_tx(&tx, run_id, step, seq)?;
        }

        tx.execute(
            "UPDATE workflow_runs SET updated_at = ?2 WHERE id = ?1",
            params![
                run_id,
                step.finished_at.as_deref().unwrap_or(&step.started_at)
            ],
        )
        .map_err(|e| e.to_string())?;

        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 追加 workflow 事件审计记录，返回自增 event id。
    pub fn append_event(
        &self,
        run_id: &str,
        step_id: Option<&str>,
        event_type: &str,
        payload: &Value,
    ) -> Result<i64, String> {
        let payload_json = serde_json::to_string(payload).map_err(|e| e.to_string())?;
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO workflow_events (run_id, step_id, event_type, payload_json, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![run_id, step_id, event_type, payload_json, now_ts()],
        )
        .map_err(|e| e.to_string())?;
        Ok(conn.last_insert_rowid())
    }

    /// 按 run id 加载完整运行态（含 steps，按 seq 排序）。
    pub fn load_run(&self, run_id: &str) -> Result<Option<WorkflowRun>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let run_row = conn.query_row(
            "SELECT definition_id, status, current_node_id, context_json,
                    session_id, error, created_at, updated_at
             FROM workflow_runs WHERE id = ?1",
            params![run_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                ))
            },
        );

        let (
            definition_id,
            status_raw,
            current_node_id,
            context_json,
            session_id,
            error,
            created_at,
            updated_at,
        ) = match run_row {
            Ok(row) => row,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(e) => return Err(e.to_string()),
        };

        let status = run_status_from_str(&status_raw)
            .ok_or_else(|| format!("unknown run status: {status_raw}"))?;
        let context: WorkflowContext =
            serde_json::from_str(&context_json).map_err(|e| e.to_string())?;

        let mut step_stmt = conn
            .prepare(
                "SELECT id, node_id, node_kind, status, started_at, finished_at,
                        input_json, output_json, error
                 FROM workflow_steps WHERE run_id = ?1 ORDER BY seq ASC",
            )
            .map_err(|e| e.to_string())?;

        let steps: Vec<StepRecord> = step_stmt
            .query_map(params![run_id], |row| {
                let node_kind_raw: String = row.get(2)?;
                let status_raw: String = row.get(3)?;
                let input_json: Option<String> = row.get(6)?;
                let output_json: Option<String> = row.get(7)?;
                Ok(StepRecord {
                    id: row.get(0)?,
                    node_id: row.get(1)?,
                    node_kind: node_kind_from_str(&node_kind_raw).unwrap_or(NodeKind::AgentReply),
                    status: step_status_from_str(&status_raw).unwrap_or(StepStatus::Queued),
                    started_at: row.get(4)?,
                    finished_at: row.get(5)?,
                    input: input_json.and_then(|s| serde_json::from_str(&s).ok()),
                    output: output_json.and_then(|s| serde_json::from_str(&s).ok()),
                    error: row.get(8)?,
                })
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        Ok(Some(WorkflowRun {
            id: run_id.to_string(),
            definition_id,
            status,
            current_node_id,
            context,
            steps,
            created_at,
            updated_at,
            session_id,
            error,
        }))
    }

    fn insert_step_tx(
        tx: &rusqlite::Transaction<'_>,
        run_id: &str,
        step: &StepRecord,
        seq: i64,
    ) -> Result<(), String> {
        let input_json = step
            .input
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|e| e.to_string())?;
        let output_json = step
            .output
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|e| e.to_string())?;

        tx.execute(
            "INSERT INTO workflow_steps
                (id, run_id, node_id, node_kind, status, started_at, finished_at,
                 input_json, output_json, error, seq)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                step.id,
                run_id,
                step.node_id,
                node_kind_to_str(step.node_kind),
                step_status_to_str(step.status),
                step.started_at,
                step.finished_at,
                input_json,
                output_json,
                step.error,
                seq,
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn update_step_tx(tx: &rusqlite::Transaction<'_>, step: &StepRecord) -> Result<(), String> {
        let output_json = step
            .output
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|e| e.to_string())?;

        tx.execute(
            "UPDATE workflow_steps SET
                status = ?2,
                finished_at = ?3,
                output_json = ?4,
                error = ?5
             WHERE id = ?1",
            params![
                step.id,
                step_status_to_str(step.status),
                step.finished_at,
                output_json,
                step.error,
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// 打开 workflow 存储并写调试日志（供 executor 复用）。
pub fn open_workflow_store(workspace: &Path) -> Result<Arc<WorkflowStore>, String> {
    let db_path = db_path_for_workspace(workspace);
    let store = Arc::new(WorkflowStore::open(&db_path)?);
    debug!("[WorkflowStore] opened {}", db_path.display());
    Ok(store)
}
