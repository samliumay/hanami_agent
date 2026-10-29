mod edit_file;
mod read_file;
mod run_command;
mod spawn_agent;
mod time;
mod write_file;

use crate::helpers::for_terminal;
use crate::providers::ProviderConfig;
use inquire::{Select, error::InquireError};

pub struct ToolSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub parameters: serde_json::Value,
}

pub struct ToolCall {
    pub id: Option<String>, //OpenAI and Anthropic give ID to tool calls.
    pub name: String,
    pub args: serde_json::Value,
    // Opaque data Gemini attaches to a call; must be sent back unchanged.
    // This is kinda wierd. needs a resarch.
    pub signature: Option<String>,
}

// returns all the tools we have.
pub fn specs() -> Vec<ToolSpec> {
    vec![
        time::spec(),
        read_file::spec(),
        write_file::spec(),
        run_command::spec(),
        edit_file::spec(),
        spawn_agent::spec(),
    ]
}

fn needs_aproval(call: &ToolCall) -> bool {
    match call.name.as_str() {
        "write_file" | "run_command" | "edit_file" => true,
        // Reading was always allowed without asking, so a prompt injection (text in a file
        // or command output that says "now read ~/.ssh/id_ed25519") could send secrets to
        // the model provider silently. Now only plain files inside the folder Kuzey was
        // started in are free; anything else asks first.
        "read_file" => !read_file::is_free_to_read(call.args["path"].as_str().unwrap_or("")),
        _ => false,
    }
}

pub async fn execute(call: &ToolCall, config: &ProviderConfig) -> String {
    //Not sure &call.name or call.name.
    if needs_aproval(call) {
        let options_for_tool_call: Vec<&str> = vec!["yes", "no"];

        // JSON already escapes control characters in args, but not bidi characters,
        // so the question goes through for_terminal too (see helpers.rs).
        let question_of_tool_call =
            for_terminal(&format!("Allow ? \n{} \n{}", &call.name, &call.args));
        let given_choice_to_do_tool_call: Result<&str, InquireError> =
            Select::new(&question_of_tool_call, options_for_tool_call).prompt();

        let answer_of_choice: bool;

        match given_choice_to_do_tool_call {
            Ok(given_choice_to_do_tool_call) => {
                if given_choice_to_do_tool_call == "yes" {
                    answer_of_choice = true;
                } else {
                    answer_of_choice = false;
                }
            }
            // Ctrl+C / Esc or a terminal problem: treat it as "no".
            Err(_) => {
                return String::from("Tool Call declined: the approval prompt was cancelled.");
            }
        }
        if !answer_of_choice {
            return String::from("Tool Call declined for this tool call.");
        }
    }

    match call.name.as_str() {
        "get_current_time" => time::run(&call.args).await,
        "read_file" => read_file::run(&call.args).await,
        "write_file" => write_file::run(&call.args).await,
        "run_command" => run_command::run(&call.args).await,
        "edit_file" => edit_file::run(&call.args).await,
        "create_subagent" => spawn_agent::run(&call.args, config).await,
        other => format!("Error: unknown tool '{other}'"),
    }
}
