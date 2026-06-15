//! Vendor / bot type constants (mirrors `common/const.py`).

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// `bot_type` in config — maps to Python `create_bot()` branches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BotType {
    #[serde(alias = "openAI")]
    OpenAi,
    #[serde(alias = "openai")]
    Openai,
    #[serde(alias = "chatGPT")]
    ChatGpt,
    Baidu,
    Qianfan,
    #[serde(alias = "xunfei")]
    Xunfei,
    #[serde(alias = "chatGPTOnAzure")]
    ChatGptOnAzure,
    #[serde(alias = "linkai")]
    Linkai,
    #[serde(alias = "claudeAPI")]
    ClaudeApi,
    #[serde(alias = "qwen")]
    Qwen,
    #[serde(alias = "dashscope")]
    QwenDashscope,
    Gemini,
    #[serde(alias = "zhipu", alias = "glm-4")]
    ZhipuAi,
    Moonshot,
    #[serde(alias = "minimax")]
    Minimax,
    Deepseek,
    #[serde(alias = "ollama")]
    Ollama,
    Custom,
    Modelscope,
    Doubao,
}

impl BotType {
    pub const OPEN_AI: &'static str = "openAI";
    pub const OPENAI: &'static str = "openai";
    pub const CHATGPT: &'static str = "chatGPT";
    pub const BAIDU: &'static str = "baidu";
    pub const QIANFAN: &'static str = "qianfan";
    pub const XUNFEI: &'static str = "xunfei";
    pub const CHATGPTONAZURE: &'static str = "chatGPTOnAzure";
    pub const LINKAI: &'static str = "linkai";
    pub const CLAUDEAPI: &'static str = "claudeAPI";
    pub const QWEN: &'static str = "qwen";
    pub const QWEN_DASHSCOPE: &'static str = "dashscope";
    pub const GEMINI: &'static str = "gemini";
    pub const ZHIPU_AI: &'static str = "zhipu";
    pub const MOONSHOT: &'static str = "moonshot";
    pub const MINIMAX: &'static str = "minimax";
    pub const DEEPSEEK: &'static str = "deepseek";
    pub const OLLAMA: &'static str = "ollama";
    pub const CUSTOM: &'static str = "custom";
    pub const MODELSCOPE: &'static str = "modelscope";
    pub const DOUBAO: &'static str = "doubao";

    pub const DEEPSEEK_V4_FLASH: &'static str = "deepseek-v4-flash";
    pub const DEEPSEEK_V4_PRO: &'static str = "deepseek-v4-pro";
    pub const DEEPSEEK_CHAT: &'static str = "deepseek-chat";
    pub const DEEPSEEK_REASONER: &'static str = "deepseek-reasoner";
}

impl fmt::Display for BotType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl BotType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OpenAi => Self::OPEN_AI,
            Self::Openai | Self::ChatGpt => Self::OPENAI,
            Self::Baidu => Self::BAIDU,
            Self::Qianfan => Self::QIANFAN,
            Self::Xunfei => Self::XUNFEI,
            Self::ChatGptOnAzure => Self::CHATGPTONAZURE,
            Self::Linkai => Self::LINKAI,
            Self::ClaudeApi => Self::CLAUDEAPI,
            Self::Qwen => Self::QWEN,
            Self::QwenDashscope => Self::QWEN_DASHSCOPE,
            Self::Gemini => Self::GEMINI,
            Self::ZhipuAi => Self::ZHIPU_AI,
            Self::Moonshot => Self::MOONSHOT,
            Self::Minimax => Self::MINIMAX,
            Self::Deepseek => Self::DEEPSEEK,
            Self::Ollama => Self::OLLAMA,
            Self::Custom => Self::CUSTOM,
            Self::Modelscope => Self::MODELSCOPE,
            Self::Doubao => Self::DOUBAO,
        }
    }
}

impl FromStr for BotType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "openAI" | "open_ai" => Ok(Self::OpenAi),
            "openai" | "chatGPT" | "chatgpt" => Ok(Self::Openai),
            "baidu" => Ok(Self::Baidu),
            "qianfan" => Ok(Self::Qianfan),
            "xunfei" => Ok(Self::Xunfei),
            "chatGPTOnAzure" | "azure" => Ok(Self::ChatGptOnAzure),
            "linkai" => Ok(Self::Linkai),
            "claudeAPI" | "claudeapi" => Ok(Self::ClaudeApi),
            "qwen" => Ok(Self::Qwen),
            "dashscope" => Ok(Self::QwenDashscope),
            "gemini" => Ok(Self::Gemini),
            "zhipu" | "glm-4" => Ok(Self::ZhipuAi),
            "moonshot" => Ok(Self::Moonshot),
            "minimax" => Ok(Self::Minimax),
            "deepseek" => Ok(Self::Deepseek),
            "ollama" => Ok(Self::Ollama),
            "custom" => Ok(Self::Custom),
            "modelscope" => Ok(Self::Modelscope),
            "doubao" => Ok(Self::Doubao),
            other => Err(format!("unknown bot_type: {other}")),
        }
    }
}
