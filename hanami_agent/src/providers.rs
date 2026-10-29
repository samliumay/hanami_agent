mod anthropic;
mod deepseek;
mod glm;
mod google;
mod kimi;
mod ollama;
mod openai;

use crate::tools::{ToolCall, ToolSpec};

//Need to check did I wrote them correctly! (new to rust)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProviderKind {
    Ollama,
    Anthropic,
    Google,
    OpenAI,
    Deepseek,
    GLM,
    Kimi,
}

impl ProviderKind {
    pub const ALL: [ProviderKind; 7] = [
        Self::Ollama,
        Self::Anthropic,
        Self::Google,
        Self::OpenAI,
        Self::Deepseek,
        Self::GLM,
        Self::Kimi,
    ];

    /// Models offered in the menu for this provider.
    pub fn models(&self) -> Vec<&'static str> {
        match self {
            Self::Ollama => ollama::all_available_models(),
            Self::Anthropic => anthropic::all_available_models(),
            Self::Google => google::all_available_models(),
            Self::OpenAI => openai::all_available_models(),
            Self::Deepseek => deepseek::all_available_models(),
            Self::GLM => glm::all_available_models(),
            Self::Kimi => kimi::all_available_models(),
        }
    }

    /// Ollama runs locally and has no key.
    pub fn needs_api_key(&self) -> bool {
        //Not sure about this syntax. Why ! at the begining of matches.
        !matches!(self, Self::Ollama)
    }
}

//Not sure what this means. Seems likea addinga function to standard format display livrary but
//what is for provider kind? Is it specilized function implementeation when its implemented for
//ProviderKind? Also need to resarch &self, f: &mut std:fmt:Formatter<'_> what this means. it
//barrows self.? then taken ownersgip of std:fmt:Formatter ? but what this means. than it returns
//result enum I think. ?
impl std::fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Ollama => "ollama",
            Self::Anthropic => "anthropic",
            Self::Google => "google",
            Self::OpenAI => "openai",
            Self::Deepseek => "deepseek",
            Self::GLM => "glm",
            Self::Kimi => "kimi",
        };
        write!(f, "{name}")
    }
}

#[derive(Clone)]
pub struct ProviderConfig {
    pub kind: ProviderKind,
    pub model: String,
    pub temp: f32,
    pub top_p: f32,
    pub top_k: u32,
    pub api_key: Option<String>,
    pub system_prompt: String,
}

impl ProviderConfig {
    pub fn new(kind: ProviderKind, model: String, api_key: Option<String>) -> Self {
        Self {
            kind,
            model,
            temp: 0.7,
            top_p: 0.95,
            top_k: 40,
            api_key,
            system_prompt: String::from(
                "You are Kuzey, a helpfull terminal agent/assistant. \
                                        Use the available tools when they help, e.g. get_current time when time is needed and so on.",
            ),
        }
    }
}

pub enum Reply {
    Text(String),
    // reasoning: the model's thinking for this whole turn (Deepseek must get it back).
    ToolCalls {
        calls: Vec<ToolCall>,
        reasoning: Option<String>,
    },
}

pub enum ChatMessage {
    User(String),
    Assistant(String),
    ToolCalls {
        calls: Vec<ToolCall>,
        reasoning: Option<String>,
    },
    ToolResult {
        id: Option<String>,
        name: String,
        output: String,
    },
}

pub async fn send(
    config: &ProviderConfig,
    history: &[ChatMessage],
    tools: &[ToolSpec],
) -> Result<Reply, Box<dyn std::error::Error>> {
    match config.kind {
        ProviderKind::Google => google::send(config, history, tools).await,
        ProviderKind::Ollama => ollama::send(config, history, tools).await,
        ProviderKind::OpenAI => openai::send(config, history, tools).await,
        ProviderKind::Anthropic => anthropic::send(config, history, tools).await,
        ProviderKind::Deepseek => deepseek::send(config, history, tools).await,
        ProviderKind::GLM => glm::send(config, history, tools).await,
        ProviderKind::Kimi => kimi::send(config, history, tools).await,
    }
}
