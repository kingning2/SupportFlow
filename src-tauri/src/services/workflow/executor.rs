//! Workflow 执行器：加载定义 → 按节点执行 → 持久化 → 发事件。

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

use chrono::Utc;
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::config::{Context, ReplyType};
use crate::events::names::{WORKFLOW_RUN_FINISHED, WORKFLOW_STEP_FINISHED, WORKFLOW_STEP_STARTED};
use crate::events::payloads::{
    WorkflowRunFinishedPayload, WorkflowStepFinishedPayload, WorkflowStepStartedPayload,
};
use crate::services::bridge::BridgeRuntime;

use super::definitions::{builtin_definition, seed_context_input};
use super::store::open_workflow_store;
use super::types::{
    NodeKind, RunStatus, StepRecord, StepStatus, WorkflowContext, WorkflowDefinition, WorkflowNode,
    WorkflowRun,
};

const MAX_NODE_ATTEMPTS: u32 = 2;

static ACTIVE_RUNS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

fn active_runs() -> &'static Mutex<HashSet<String>> {
    ACTIVE_RUNS.get_or_init(|| Mutex::new(HashSet::new()))
}

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

fn render_template(template: &str, ctx: &WorkflowContext) -> String {
    let mut out = template.to_string();
    for (key, value) in &ctx.vars {
        let placeholder = format!("{{{{{key}}}}}");
        let text = match value {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        out = out.replace(&placeholder, &text);
    }
    out
}

/// 工作流执行器（MVP：单进程、线性 + 简单分支）。
pub struct WorkflowExecutor;

impl WorkflowExecutor {
    /// 启动新 run 并在后台执行至完成或阻塞节点。
    pub async fn start(
        app: AppHandle,
        workspace: PathBuf,
        bridge: Arc<BridgeRuntime>,
        definition_id: &str,
        input: Value,
        session_id: Option<String>,
    ) -> Result<String, String> {
        let definition = builtin_definition(definition_id)
            .ok_or_else(|| format!("unknown workflow definition: {definition_id}"))?;
        Self::start_with_definition(app, workspace, bridge, definition, input, session_id).await
    }

    /// 使用完整定义启动 run。
    pub async fn start_with_definition(
        app: AppHandle,
        workspace: PathBuf,
        bridge: Arc<BridgeRuntime>,
        definition: WorkflowDefinition,
        input: Value,
        session_id: Option<String>,
    ) -> Result<String, String> {
        let store = open_workflow_store(&workspace)?;
        let run_id = Uuid::new_v4().to_string();
        let now = now_rfc3339();
        let mut context = seed_context_input(&input);
        context
            .vars
            .insert("definition_id".into(), json!(definition.id));

        let run = WorkflowRun {
            id: run_id.clone(),
            definition_id: definition.id.clone(),
            status: RunStatus::Pending,
            current_node_id: Some(definition.entry_node_id.clone()),
            context,
            steps: vec![],
            created_at: now.clone(),
            updated_at: now,
            session_id: session_id.clone(),
            error: None,
        };
        store.create_run(&run)?;

        let app_bg = app.clone();
        let definition_id = definition.id.clone();
        let run_id_spawn = run_id.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(e) =
                Self::execute_run(app_bg, workspace, bridge, definition, run_id_spawn).await
            {
                error!("[WorkflowExecutor] run failed definition={definition_id}: {e}");
            }
        });

        Ok(run_id)
    }

    /// 从持久化状态恢复并继续执行。
    pub async fn resume(
        app: AppHandle,
        workspace: PathBuf,
        bridge: Arc<BridgeRuntime>,
        run_id: &str,
    ) -> Result<(), String> {
        let store = open_workflow_store(&workspace)?;
        let run = store
            .load_run(run_id)?
            .ok_or_else(|| format!("workflow run not found: {run_id}"))?;

        match run.status {
            RunStatus::WaitingHuman | RunStatus::Paused | RunStatus::Running => {}
            RunStatus::Pending => {}
            other => {
                return Err(format!(
                    "run {run_id} cannot resume from status {:?}",
                    other
                ));
            }
        }

        let definition = builtin_definition(&run.definition_id)
            .ok_or_else(|| format!("definition {} not found", run.definition_id))?;

        let app_bg = app.clone();
        let run_id_owned = run_id.to_string();
        tauri::async_runtime::spawn(async move {
            if let Err(e) =
                Self::execute_run(app_bg, workspace, bridge, definition, run_id_owned).await
            {
                error!("[WorkflowExecutor] resume failed: {e}");
            }
        });
        Ok(())
    }

    async fn execute_run(
        app: AppHandle,
        workspace: PathBuf,
        bridge: Arc<BridgeRuntime>,
        definition: WorkflowDefinition,
        run_id: String,
    ) -> Result<(), String> {
        {
            let mut guard = active_runs().lock().map_err(|e| e.to_string())?;
            if !guard.insert(run_id.clone()) {
                warn!("[WorkflowExecutor] run {} already active", run_id);
                return Ok(());
            }
        }

        let result = Self::run_loop(&app, &workspace, &bridge, &definition, &run_id).await;

        {
            let mut guard = active_runs().lock().map_err(|e| e.to_string())?;
            guard.remove(&run_id);
        }

        result
    }

    async fn run_loop(
        app: &AppHandle,
        workspace: &PathBuf,
        bridge: &Arc<BridgeRuntime>,
        definition: &WorkflowDefinition,
        run_id: &str,
    ) -> Result<(), String> {
        let store = open_workflow_store(workspace)?;

        loop {
            let mut run = store
                .load_run(run_id)?
                .ok_or_else(|| format!("run missing: {run_id}"))?;

            if matches!(
                run.status,
                RunStatus::Succeeded | RunStatus::Failed | RunStatus::Cancelled
            ) {
                return Ok(());
            }

            let node_id = run
                .current_node_id
                .clone()
                .ok_or_else(|| "run has no current node".to_string())?;
            let node = definition
                .nodes
                .iter()
                .find(|n| n.id == node_id)
                .ok_or_else(|| format!("node {node_id} not in definition"))?
                .clone();

            run.status = RunStatus::Running;
            run.updated_at = now_rfc3339();
            store.update_run(&run)?;

            let step_id = Uuid::new_v4().to_string();
            let step_started = StepRecord {
                id: step_id.clone(),
                node_id: node_id.clone(),
                node_kind: node.kind,
                status: StepStatus::Active,
                started_at: now_rfc3339(),
                finished_at: None,
                input: Some(json!({"node": node_id})),
                output: None,
                error: None,
            };
            store.update_step(run_id, &step_started, run.steps.len() as i64)?;
            store.append_event(run_id, Some(&step_id), "step_started", &json!({}))?;

            let _ = app.emit(
                WORKFLOW_STEP_STARTED,
                WorkflowStepStartedPayload {
                    run_id: run_id.to_string(),
                    step_id: step_id.clone(),
                    node_id: node_id.clone(),
                    node_type: format!("{:?}", node.kind),
                },
            );

            let exec_result = Self::execute_node_with_retry(bridge, &node, &run).await;

            match exec_result {
                Ok(output) => {
                    let mut finished = step_started.clone();
                    finished.status = StepStatus::Completed;
                    finished.finished_at = Some(now_rfc3339());
                    finished.output = Some(output.clone());
                    store.update_step(run_id, &finished, run.steps.len() as i64)?;
                    store.append_event(run_id, Some(&step_id), "step_finished", &output)?;

                    let _ = app.emit(
                        WORKFLOW_STEP_FINISHED,
                        WorkflowStepFinishedPayload {
                            run_id: run_id.to_string(),
                            step_id: step_id.clone(),
                            status: "completed".into(),
                            output: Some(output.clone()),
                            error: None,
                        },
                    );

                    run = store.load_run(run_id)?.expect("run");
                    if let Some(key) = node_output_key(&node) {
                        let stored = output.get("reply").cloned().unwrap_or(output);
                        run.context.vars.insert(key, stored);
                    }

                    if node.kind == NodeKind::HumanAndsign {
                        run.status = RunStatus::WaitingHuman;
                        run.updated_at = now_rfc3339();
                        store.update_run(&run)?;
                        return Ok(());
                    }

                    if node.kind == NodeKind::Delay {
                        run.status = RunStatus::Paused;
                        run.updated_at = now_rfc3339();
                        store.update_run(&run)?;
                        let delay_secs = node
                            .config
                            .delay
                            .as_ref()
                            .and_then(|d| d.duration_secs)
                            .unwrap_or(1) as u64;
                        tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
                        run.status = RunStatus::Running;
                        run.updated_at = now_rfc3339();
                        store.update_run(&run)?;
                    }

                    match resolve_next_node(definition, &node_id, &run.context) {
                        Some(next) => {
                            run.current_node_id = Some(next);
                            run.updated_at = now_rfc3339();
                            store.update_run(&run)?;
                        }
                        None => {
                            run.status = RunStatus::Succeeded;
                            run.current_node_id = None;
                            run.updated_at = now_rfc3339();
                            store.update_run(&run)?;
                            let _ = app.emit(
                                WORKFLOW_RUN_FINISHED,
                                WorkflowRunFinishedPayload {
                                    run_id: run_id.to_string(),
                                    status: "succeeded".into(),
                                    error: None,
                                },
                            );
                            info!("[WorkflowExecutor] run {run_id} succeeded");
                            return Ok(());
                        }
                    }
                }
                Err(e) => {
                    let mut failed = step_started;
                    failed.status = StepStatus::Failed;
                    failed.finished_at = Some(now_rfc3339());
                    failed.error = Some(e.clone());
                    store.update_step(run_id, &failed, run.steps.len() as i64)?;

                    let _ = app.emit(
                        WORKFLOW_STEP_FINISHED,
                        WorkflowStepFinishedPayload {
                            run_id: run_id.to_string(),
                            step_id,
                            status: "failed".into(),
                            output: None,
                            error: Some(e.clone()),
                        },
                    );

                    run = store.load_run(run_id)?.expect("run");
                    run.status = RunStatus::Failed;
                    run.error = Some(e.clone());
                    run.updated_at = now_rfc3339();
                    store.update_run(&run)?;

                    let _ = app.emit(
                        WORKFLOW_RUN_FINISHED,
                        WorkflowRunFinishedPayload {
                            run_id: run_id.to_string(),
                            status: "failed".into(),
                            error: Some(e),
                        },
                    );
                    return Ok(());
                }
            }
        }
    }

    async fn execute_node_with_retry(
        bridge: &Arc<BridgeRuntime>,
        node: &WorkflowNode,
        run: &WorkflowRun,
    ) -> Result<Value, String> {
        let mut last_err = String::new();
        for attempt in 0..MAX_NODE_ATTEMPTS {
            match Self::execute_node(bridge, node, run).await {
                Ok(v) => return Ok(v),
                Err(e) => {
                    last_err = e;
                    if attempt + 1 < MAX_NODE_ATTEMPTS {
                        warn!(
                            "[WorkflowExecutor] node {} attempt {} failed, retrying",
                            node.id,
                            attempt + 1
                        );
                    }
                }
            }
        }
        Err(last_err)
    }

    async fn execute_node(
        bridge: &Arc<BridgeRuntime>,
        node: &WorkflowNode,
        run: &WorkflowRun,
    ) -> Result<Value, String> {
        match node.kind {
            NodeKind::AgentReply => Self::handle_agent_reply(bridge, node, run).await,
            NodeKind::ToolCall => Self::handle_tool_call(bridge, node, run).await,
            NodeKind::Branch => Self::handle_branch(node, run),
            NodeKind::HumanAndsign => Self::handle_human_andsign(node),
            NodeKind::Delay => Ok(json!({"delayed": true})),
            NodeKind::DelegateToRole => Self::handle_delegate_to_role(bridge, node, run).await,
        }
    }

    async fn handle_delegate_to_role(
        bridge: &Arc<BridgeRuntime>,
        node: &WorkflowNode,
        run: &WorkflowRun,
    ) -> Result<Value, String> {
        let cfg = node
            .config
            .delegate_to_role
            .as_ref()
            .ok_or_else(|| "delegate_to_role config missing".to_string())?;
        let binding = cfg.binding.clone().unwrap_or_else(|| match cfg.role {
            crate::services::agent::AgentRole::Planner => {
                crate::services::agent::RoleBinding::planner_default()
            }
            crate::services::agent::AgentRole::Executor => {
                crate::services::agent::RoleBinding::executor_default()
            }
            crate::services::agent::AgentRole::Reviewer => {
                crate::services::agent::RoleBinding::reviewer_default()
            }
        });
        let mut prompt = render_template(&cfg.prompt_template, &run.context);
        if let Some(suffix) = &binding.system_prompt_suffix {
            prompt = format!("{prompt}\n\n{suffix}");
        }

        let mut ctx = Context::default();
        if let Some(sid) = &run.session_id {
            ctx.set("session_id", sid);
        }
        ctx.set("request_id", &format!("wf-{}-{}", run.id, node.id));
        ctx.set("channel_type", "web");
        ctx.set("workflow_run_id", &run.id);
        ctx.set("agent_role", binding.role.as_str());

        let reply = bridge
            .agent_bridge
            .agent_reply(&prompt, Some(ctx), None, false)
            .await;

        if reply.ty == ReplyType::Error {
            return Err(reply.content);
        }
        Ok(json!({"reply": reply.content, "role": binding.role.as_str()}))
    }

    async fn handle_agent_reply(
        bridge: &Arc<BridgeRuntime>,
        node: &WorkflowNode,
        run: &WorkflowRun,
    ) -> Result<Value, String> {
        let cfg = node
            .config
            .agent_reply
            .as_ref()
            .ok_or_else(|| "agent_reply config missing".to_string())?;
        let prompt = render_template(&cfg.prompt_template, &run.context);

        let mut ctx = Context::default();
        if let Some(sid) = &run.session_id {
            ctx.set("session_id", sid);
        }
        ctx.set("request_id", &format!("wf-{}", run.id));
        ctx.set("channel_type", "web");

        let reply = bridge
            .agent_bridge
            .agent_reply(&prompt, Some(ctx), None, cfg.clear_history)
            .await;

        if reply.ty == ReplyType::Error {
            return Err(reply.content);
        }
        let text = reply.content;
        Ok(json!({"reply": text}))
    }

    async fn handle_tool_call(
        bridge: &Arc<BridgeRuntime>,
        node: &WorkflowNode,
        run: &WorkflowRun,
    ) -> Result<Value, String> {
        let cfg = node
            .config
            .tool_call
            .as_ref()
            .ok_or_else(|| "tool_call config missing".to_string())?;
        let agent = bridge
            .agent_bridge
            .ensure_agent(run.session_id.as_deref(), "web")?;
        let tool = agent
            .tools
            .iter()
            .find(|t| t.name() == cfg.tool_name)
            .ok_or_else(|| format!("tool not found: {}", cfg.tool_name))?
            .clone();
        let result = tool.execute(cfg.arguments.clone()).await;
        if result.status != "success" {
            return Err(format!(
                "tool {} failed: {:?}",
                cfg.tool_name, result.result
            ));
        }
        Ok(result.result)
    }

    fn handle_branch(node: &WorkflowNode, run: &WorkflowRun) -> Result<Value, String> {
        let cfg = node
            .config
            .branch
            .as_ref()
            .ok_or_else(|| "branch config missing".to_string())?;
        let flag = run
            .context
            .vars
            .get(&cfg.condition_key)
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        Ok(json!({"branch": flag}))
    }

    fn handle_human_andsign(node: &WorkflowNode) -> Result<Value, String> {
        let cfg = node
            .config
            .human_andsign
            .as_ref()
            .ok_or_else(|| "human_andsign config missing".to_string())?;
        Ok(json!({"prompt": cfg.prompt}))
    }
}

fn resolve_next_node(
    definition: &WorkflowDefinition,
    current: &str,
    context: &WorkflowContext,
) -> Option<String> {
    let outgoing: Vec<_> = definition
        .transitions
        .iter()
        .filter(|t| t.from == current)
        .collect();
    if outgoing.is_empty() {
        return None;
    }
    for t in &outgoing {
        if let Some(cond_key) = &t.condition {
            if context.vars.get(cond_key).and_then(|v| v.as_bool()) == Some(true) {
                return Some(t.to.clone());
            }
        }
    }
    outgoing
        .iter()
        .find(|t| t.condition.is_none())
        .map(|t| t.to.clone())
        .or_else(|| outgoing.first().map(|t| t.to.clone()))
}

fn node_output_key(node: &WorkflowNode) -> Option<String> {
    match node.kind {
        NodeKind::AgentReply => node
            .config
            .agent_reply
            .as_ref()
            .and_then(|c| c.output_key.clone()),
        NodeKind::ToolCall => node
            .config
            .tool_call
            .as_ref()
            .and_then(|c| c.output_key.clone()),
        NodeKind::DelegateToRole => node
            .config
            .delegate_to_role
            .as_ref()
            .and_then(|c| c.output_key.clone()),
        _ => None,
    }
}

/// 解析下一节点（供测试使用）。
#[doc(hidden)]
pub fn resolve_next_node_for_test(
    definition: &WorkflowDefinition,
    current: &str,
    context: &WorkflowContext,
) -> Option<String> {
    resolve_next_node(definition, current, context)
}
