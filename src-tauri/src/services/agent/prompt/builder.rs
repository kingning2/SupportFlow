//! `agent/prompt/builder.py`

use std::path::Path;

use chrono::Local;

use crate::services::agent::skills::SkillManager;
use crate::services::agent::tools::AgentTool;

use super::types::ContextFile;
use super::workspace::load_context_files;

pub struct PromptBuilder {
    pub workspace_dir: std::path::PathBuf,
    pub language: String,
}

impl PromptBuilder {
    pub fn new(workspace_dir: impl AsRef<Path>, language: impl Into<String>) -> Self {
        Self {
            workspace_dir: workspace_dir.as_ref().to_path_buf(),
            language: language.into(),
        }
    }

    pub fn build(
        &self,
        tools: &[std::sync::Arc<dyn AgentTool>],
        skill_manager: Option<&SkillManager>,
        skill_filter: Option<&[String]>,
        include_memory_section: bool,
        include_knowledge: bool,
        runtime_model: Option<&str>,
    ) -> String {
        build_agent_system_prompt(
            &self.workspace_dir,
            tools,
            skill_manager,
            skill_filter,
            include_memory_section,
            include_knowledge,
            runtime_model,
        )
    }
}

pub fn build_agent_system_prompt(
    workspace_dir: &Path,
    tools: &[std::sync::Arc<dyn AgentTool>],
    skill_manager: Option<&SkillManager>,
    skill_filter: Option<&[String]>,
    include_memory_section: bool,
    include_knowledge: bool,
    runtime_model: Option<&str>,
) -> String {
    let mut sections: Vec<String> = Vec::new();

    if !tools.is_empty() {
        sections.extend(build_tooling_section(tools));
    }

    if let Some(sm) = skill_manager {
        sections.extend(build_skills_section(sm, tools, skill_filter));
    }

    if include_memory_section {
        sections.extend(build_memory_section(workspace_dir));
    }

    if include_knowledge {
        sections.extend(build_knowledge_section(workspace_dir));
    }

    sections.extend(build_workspace_section(workspace_dir));

    let context_files = load_context_files(workspace_dir, None);
    if !context_files.is_empty() {
        sections.extend(build_context_files_section(&context_files));
    }

    if let Some(model) = runtime_model {
        sections.extend(build_runtime_section(model));
    }

    sections.join("\n")
}

fn build_tooling_section(tools: &[std::sync::Arc<dyn AgentTool>]) -> Vec<String> {
    let summaries: &[(&str, &str)] = &[
        ("read", "读取文件内容"),
        ("write", "创建或覆盖文件"),
        ("edit", "精确编辑文件"),
        ("ls", "列出目录内容"),
        ("bash", "执行shell命令"),
        ("memory_search", "搜索记忆"),
        ("memory_get", "读取记忆内容"),
        ("send", "发送本地文件给用户"),
        ("env_config", "管理API密钥和技能配置"),
        ("web_search", "网络搜索"),
        ("web_fetch", "获取网页/文档内容"),
        ("browser", "控制浏览器（Chromium/CDP）"),
        ("vision", "图像理解/视觉问答"),
    ];

    let mut tool_lines = Vec::new();
    for tool in tools {
        let name = tool.name();
        let summary = summaries
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, s)| *s)
            .unwrap_or("");
        if summary.is_empty() {
            tool_lines.push(format!("- {name}"));
        } else {
            tool_lines.push(format!("- {name}: {summary}"));
        }
    }

    vec![
        "## 🔧 工具系统".into(),
        String::new(),
        "可用工具（名称大小写敏感，严格按列表调用）:".into(),
        tool_lines.join("\n"),
        String::new(),
        "工具调用风格：".into(),
        String::new(),
        "- 多步骤任务、复杂决策、敏感操作时，应简要说明当前在做什么、为什么这样做".into(),
        "- 持续推进直到任务完成，完成后向用户报告结果".into(),
        "- 回复中涉及密钥、令牌等敏感信息必须脱敏".into(),
        "- URL链接直接放在回复文本中即可，系统会自动处理和渲染".into(),
        String::new(),
    ]
}

fn build_skills_section(
    skill_manager: &SkillManager,
    tools: &[std::sync::Arc<dyn AgentTool>],
    skill_filter: Option<&[String]>,
) -> Vec<String> {
    let read_tool = tools
        .iter()
        .find(|t| t.name().eq_ignore_ascii_case("read"))
        .map(|t| t.name())
        .unwrap_or("read");

    let mut lines = vec![
        "## 🧩 技能系统（mandatory）".into(),
        String::new(),
        "在回复之前：扫描下方 <available_skills> 中每个技能的 <description>。".into(),
        String::new(),
        format!(
            "- 如果有技能的描述与用户需求匹配：使用 `{read_tool}` 工具读取其 <location> 路径的 SKILL.md 文件，然后严格遵循文件中的指令。"
        ),
        "- 如果多个技能都适用则选择最匹配的一个，然后读取并遵循。".into(),
        "- 如果没有技能明确适用：不要读取任何 SKILL.md，直接使用通用工具。".into(),
        String::new(),
        format!(
            "**重要**: 技能不是工具，不能直接调用。使用技能的唯一方式是用 `{read_tool}` 读取 SKILL.md 文件。"
        ),
        "永远不要一次性读取多个技能，只在选择后再读取。".into(),
        String::new(),
        "以下是可用技能：".into(),
    ];

    let prompt = skill_manager.build_skills_prompt(skill_filter);
    if !prompt.is_empty() {
        lines.push(prompt);
        lines.push(String::new());
    }

    lines
}

fn build_memory_section(workspace_dir: &Path) -> Vec<String> {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let today_file = format!("{today}.md");
    vec![
        "## 🧠 记忆系统".into(),
        String::new(),
        "### Memory Recall（mandatory）".into(),
        String::new(),
        "当用户询问过往事件、引用之前的决定、提到人物关系、偏好、待办、或你对某事不确定时，**必须先检索记忆再回答**。".into(),
        String::new(),
        "1. 不确定位置 → `memory_search` 关键词/语义检索".into(),
        "2. 已知位置 → `memory_get` 直接读取对应行".into(),
        format!("3. 每日记忆文件: memory/{today_file}").into(),
        String::new(),
        format!("工作空间: `{}`", workspace_dir.display()),
        String::new(),
    ]
}

fn build_knowledge_section(workspace_dir: &Path) -> Vec<String> {
    let index = workspace_dir.join("knowledge/index.md");
    if !index.is_file() {
        return vec![];
    }
    let Ok(content) = std::fs::read_to_string(&index) else {
        return vec![];
    };
    if content.trim().is_empty() {
        return vec![];
    }

    vec![
        "## 📚 知识系统".into(),
        String::new(),
        "你拥有一个持续积累的个人知识库 `knowledge/`。".into(),
        String::new(),
        "### knowledge/index.md".into(),
        String::new(),
        content.trim().to_string(),
        String::new(),
    ]
}

fn build_workspace_section(workspace_dir: &Path) -> Vec<String> {
    vec![
        "## 📂 工作空间".into(),
        String::new(),
        format!("你的工作目录是: `{}`", workspace_dir.display()),
        String::new(),
        "**路径使用规则**:".into(),
        format!("- 相对路径均相对于 `{}`", workspace_dir.display()),
        "- 访问工作空间外目录请使用绝对路径（如 `~/project`）".into(),
        String::new(),
        "**已自动加载的文件**（无需 read）: AGENT.md, USER.md, RULE.md, MEMORY.md".into(),
        String::new(),
    ]
}

fn build_context_files_section(files: &[ContextFile]) -> Vec<String> {
    let mut lines = vec![
        "# 📋 项目上下文".into(),
        String::new(),
        "以下项目上下文文件已被加载：".into(),
        String::new(),
    ];
    for file in files {
        lines.push(format!("## {}", file.path));
        lines.push(String::new());
        lines.push(file.content.clone());
        lines.push(String::new());
    }
    lines
}

fn build_runtime_section(model: &str) -> Vec<String> {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    vec![
        "## ⏱ 运行时".into(),
        String::new(),
        format!("- 当前时间: {now}"),
        format!("- 模型: {model}"),
        String::new(),
    ]
}
