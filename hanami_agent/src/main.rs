mod agents;
mod configuration;
mod helpers;
mod providers;
mod tools;

use configuration::ConfigurationStyle;
use helpers::{MentionCompleter, expand_mentions, for_terminal};
use inquire::{Select, Text};
use providers::{ChatMessage, ProviderConfig};

#[tokio::main]
async fn main() {
    println!("Kuzey Agent 0.1.0");

    // First-run check: ensure `.hanami/` and `.hanami/.env` exist before asking anything.
    match configuration::env_config::ensure_initialized().await {
        Ok(true) => {
            println!(
                "First run detected: created '{}'.",
                configuration::env_config::DEFAULT_ENV_PATH
            );
        }
        Ok(false) => {}
        Err(e) => {
            println!("Error initializing .hanami directory: {e}");
            return;
        }
    }

    let Ok(env_or_manual_config) = Select::new(
        "Will you configure the system by yourself or you will reference the env file.",
        ConfigurationStyle::ALL.to_vec(),
    )
    .prompt() else {
        println!("Exiting...");
        return;
    };

    let Some((kind, api_key, model)) = configuration::configure(env_or_manual_config).await else {
        println!("Exiting...");
        return;
    };

    let provider_details = ProviderConfig::new(kind, model, api_key);

    let tool_specs = tools::specs();
    let mut history: Vec<ChatMessage> = Vec::new();

    loop {
        let Ok(input) = Text::new("You:")
            .with_autocomplete(MentionCompleter)
            .prompt()
        else {
            println!("Exiting...");
            break;
        };
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "/exit" {
            println!("Exiting...");
            break;
        }

        // Where this turn starts, so a failed turn can be removed completely.
        let turn_start = history.len();
        history.push(ChatMessage::User(expand_mentions(input)));

        // Run the shared agent loop (up to 10 rounds of tool calls).
        match agents::run(&provider_details, &mut history, &tool_specs, "main", 10).await {
            Ok(text) => {
                println!("{}", for_terminal(&text));
            }
            Err(e) => {
                println!("Error {e}");
                // Drop the whole failed turn so history never ends with a dangling ToolCalls.
                history.truncate(turn_start);
            }
        }
    }
}
