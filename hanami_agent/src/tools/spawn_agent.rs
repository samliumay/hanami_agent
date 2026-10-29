//! Tool that spawns an isolated subagent (`explorer` or `coder`) to handle a subtask.
//!
//! Each subagent gets its own fresh `Vec<ChatMessage>` history, its own `system_prompt`,
//! and a filtered list of allowed tools, while reusing the parent's provider/model/API key.

use super::ToolSpec;
use crate::agents::{self, AGENTS};
use crate::providers::{ChatMessage, ProviderConfig};
use serde_json::{Value, json};

pub(super) fn spec() -> ToolSpec {
    // Collect all defined subagent names so the JSON Schema `enum` stays in sync with `AGENTS`.
    let agent_names: Vec<&str> = AGENTS.iter().map(|a| a.name).collect();
    let agent_descriptions: Vec<String> = AGENTS
        .iter()
        .map(|a| format!("'{}': {}", a.name, a.description))
        .collect();

    ToolSpec {
        name: "create_subagent",
        description: "Spawns a specialized subagent with its own isolated context to complete a subtask.",
        parameters: json!({
            "type": "object",
            "properties": {
                "agent": {
                    "type": "string",
                    "enum": agent_names,
                    "description": format!("Which specialized subagent to run ({}).", agent_descriptions.join("; "))
                },
                "user_prompt": {
                    "type": "string",
                    "description": "The task instructions and context given to the subagent."
                }
            },
            "required": ["agent", "user_prompt"]
        }),
    }
}

pub(super) async fn run(args: &Value, parent_config: &ProviderConfig) -> String {
    let Some(agent_name) = args["agent"].as_str() else {
        return String::from("Error: missing 'agent' argument.");
    };
    let Some(user_prompt) = args["user_prompt"].as_str() else {
        return String::from("Error: missing 'user_prompt' argument.");
    };

    let Some(agent_def) = agents::find_agent(agent_name) else {
        return format!("Error: unknown subagent '{agent_name}'.");
    };

    // Clone the parent's provider config, but swap in the subagent's system prompt.
    let mut sub_config = parent_config.clone();
    sub_config.system_prompt = agent_def.system_prompt.to_string();

    // Filter the full tool list down to only the tools allowed for this subagent.
    let allowed_tools: Vec<ToolSpec> = super::specs()
        .into_iter()
        .filter(|t| agent_def.tools.contains(&t.name))
        .collect();

    // Subagent starts with a clean, isolated conversation history.
    let mut sub_history = vec![ChatMessage::User(user_prompt.to_string())];

    // `Box::pin` puts the recursive async call on the heap so the Rust compiler doesn't see
    // an infinitely sized Future (`agents::run -> tools::execute -> spawn_agent::run -> agents::run`).
    match Box::pin(agents::run(
        &sub_config,
        &mut sub_history,
        &allowed_tools,
        agent_def.name,
        agent_def.max_rounds,
    ))
    .await
    {
        Ok(reply) => reply,
        Err(e) => format!("Subagent '{agent_name}' failed: {e}"),
    }
}
