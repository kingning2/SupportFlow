//! 内置工具的中文展示名称与描述（控制台 UI 与中文 Agent 提示）。

/// 返回工具在控制台展示的中文名称；未知工具回退为原始 `name`。
pub fn tool_label(name: &str) -> &str {
    match name {
        "read" => "读取文件",
        "write" => "写入文件",
        "edit" => "编辑文件",
        "bash" => "执行命令",
        "ls" => "列出目录",
        "send" => "发送文件",
        "memory_search" => "记忆检索",
        "memory_get" => "读取记忆",
        "env_config" => "环境配置",
        "web_fetch" => "网页抓取",
        "web_search" => "网页搜索",
        "browser" => "浏览器控制",
        "vision" => "图像理解",
        _ => name,
    }
}

/// 返回工具中文描述；未知工具回退为运行时英文描述。
pub fn tool_description_zh(name: &str, fallback: &str) -> String {
    let desc = match name {
        "read" => "读取或检查文件内容。文本/PDF 返回内容（超长会截断）；图片/音视频等返回元数据。",
        "write" => {
            "写入文件内容。不存在则创建，存在则覆盖；自动创建父目录。单次写入不宜超过 10KB。"
        }
        "edit" => "通过精确匹配替换编辑文件；`oldText` 为空时追加到文件末尾。",
        "bash" => "在当前工作目录执行 Shell 命令，返回标准输出与错误输出（超长会截断）。",
        "ls" => "列出目录内容，按字母排序；目录名以 `/` 结尾，包含隐藏文件。",
        "send" => "向用户发送本地文件（图片、音视频、文档）。仅用于本地路径，不要传 URL。",
        "memory_search" => "在长期记忆与知识库中进行语义/关键词检索，召回历史对话与知识页。",
        "memory_get" => "读取指定记忆或知识文件的完整内容，支持按行范围读取。",
        "env_config" => "安全管理 API 密钥与技能配置：设置、查看、列出、删除；值自动脱敏。",
        "web_fetch" => "抓取 HTTP/HTTPS 页面或文档（PDF、Word、Excel 等）并提取可读文本。",
        "web_search" => "搜索互联网实时信息，返回标题、链接与摘要。",
        "browser" => "控制浏览器：导航、快照、点击、填表、滚动、截图等（基于本机 Chrome/Edge）。",
        "vision" => "分析本地图片或图片 URL，可描述内容、提取文字、识别物体等。",
        _ => return fallback.to_string(),
    };
    desc.to_string()
}
