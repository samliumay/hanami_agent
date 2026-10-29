//! Subagent definitions and the shared agent execution loop.
//!
//! Both the main chat loop in `main.rs` and spawned subagents in `tools/spawn_agent.rs`
//! run through [`run`], which repeatedly sends the conversation history to the provider
//! and executes any requested tool calls up to `max_rounds`.

use crate::helpers::for_terminal;
use crate::providers::{self, ChatMessage, ProviderConfig, Reply};
use crate::tools::{self, ToolSpec};

/// Blueprint for a specialized subagent.
pub struct AgentDef {
    pub name: &'static str,
    pub description: &'static str,
    pub system_prompt: &'static str,
    /// Names of tools this subagent is allowed to use.
    pub tools: &'static [&'static str],
    /// Maximum tool-calling rounds before stopping.
    pub max_rounds: usize,
}

/// Built-in subagents available to the `create_subagent` tool.
///
/// Note that `"create_subagent"` is intentionally omitted from `tools` so a subagent
/// cannot recursively spawn more subagents.
pub const AGENTS: &[AgentDef] = &[
    AgentDef {
        name: "explorer",
        description: "Read-only agent for exploring files and searching the codebase.",
        system_prompt: "You are a codebase explorer subagent. Read files to answer the questions concisely. Do not modify files.",
        tools: &["read_file", "get_current_time"],
        max_rounds: 6,
    },
    AgentDef {
        name: "coder",
        description: "Subagent that can read, write, edit files, and run commands to complete a coding task.",
        system_prompt: "You are a coding subagent. Complete the given task using your tools and return a concise summary of what you did.",
        tools: &[
            "read_file",
            "write_file",
            "edit_file",
            "run_command",
            "get_current_time",
        ],
        max_rounds: 10,
    },
];

/// Looks up a subagent definition by name.
pub fn find_agent(name: &str) -> Option<&'static AgentDef> {
    AGENTS.iter().find(|a| a.name == name)
}

/// Runs the agent tool loop up to `max_rounds` times until the model replies with plain text.
pub async fn run(
    config: &ProviderConfig,
    history: &mut Vec<ChatMessage>,
    tools: &[ToolSpec],
    label: &str,
    max_rounds: usize,
) -> Result<String, Box<dyn std::error::Error>> {
    for _ in 0..max_rounds {
        match providers::send(config, history, tools).await? {
            Reply::Text(text) => {
                history.push(ChatMessage::Assistant(text.clone()));
                return Ok(text);
            }
            Reply::ToolCalls { calls, reasoning } => {
                let mut results = Vec::new();
                for call in &calls {
                    println!(
                        "[{label} -> tool] {} {}",
                        for_terminal(&call.name),
                        call.args
                    );
                    let output = tools::execute(call, config).await;
                    results.push(ChatMessage::ToolResult {
                        id: call.id.clone(),
                        name: call.name.clone(),
                        output,
                    });
                }
                history.push(ChatMessage::ToolCalls { calls, reasoning });
                history.extend(results);
            }
        }
    }
    Err("Ran out of tool rounds".into())
}
