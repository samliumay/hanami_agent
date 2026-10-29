//! Reads provider, API key, and model configuration from a `.env` file.
//!
//! - On the very first run, Hanami checks whether `.hanami/.env` exists. If not, it creates
//!   `.hanami/` and an empty `.hanami/.env` template.
//! - When loading the `.env` file, if the required keys (`PROVIDER`, `MODEL`, or `API_KEY`)
//!   are missing or empty, Hanami prompts the user to choose a provider/key/model and saves
//!   the answers back into the `.env` file for future runs.

use super::manual_config;
use crate::providers::ProviderKind;
use inquire::{Select, Text};
use std::path::Path;

/// Default location for the environment configuration file inside the workspace.
pub const DEFAULT_ENV_PATH: &str = ".hanami/.env";

/// Empty template written when `.hanami/.env` is first created.
const EMPTY_ENV_TEMPLATE: &str = "\
# Hanami Agent Configuration
# Available providers: ollama, anthropic, google, openai, deepseek, glm, kimi
PROVIDER=
MODEL=
API_KEY=
";

/// Formats the chosen `(kind, api_key, model)` into `.env` file content.
fn format_env_content(kind: ProviderKind, api_key: Option<&str>, model: &str) -> String {
    let key_str = api_key.unwrap_or("");
    format!(
        "# Hanami Agent Configuration\n\
         # Available providers: ollama, anthropic, google, openai, deepseek, glm, kimi\n\
         PROVIDER={kind}\n\
         MODEL={model}\n\
         API_KEY={key_str}\n"
    )
}

/// Checks at program startup whether `.hanami/.env` exists to detect the first run.
///
/// - **First run** (`.hanami/.env` does not exist): creates `.hanami/` and writes an empty
///   `.hanami/.env` template, then returns `Ok(true)`.
/// - **Subsequent runs** (`.hanami/.env` already exists): does nothing and returns `Ok(false)`.
pub async fn ensure_initialized() -> std::io::Result<bool> {
    if tokio::fs::try_exists(DEFAULT_ENV_PATH).await? {
        return Ok(false); // Not the first run; file already exists.
    }

    tokio::fs::create_dir_all(".hanami").await?;
    tokio::fs::write(DEFAULT_ENV_PATH, EMPTY_ENV_TEMPLATE).await?;
    Ok(true)
}

/// Ensures that a given `.env` file path exists, creating its parent folders and
/// writing [`EMPTY_ENV_TEMPLATE`] if the file is missing.
async fn create_env_file_if_missing(path: &str) -> std::io::Result<()> {
    if !tokio::fs::try_exists(path).await? {
        if let Some(parent) = Path::new(path).parent()
            && !parent.as_os_str().is_empty()
        {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(path, EMPTY_ENV_TEMPLATE).await?;
    }
    Ok(())
}

/// Converts a raw string from the `.env` file (e.g. `"Google"` or `"ollama"`)
/// into our [`ProviderKind`] enum.
fn parse_provider(value: &str) -> Option<ProviderKind> {
    match value.trim().to_lowercase().as_str() {
        "ollama" => Some(ProviderKind::Ollama),
        "anthropic" => Some(ProviderKind::Anthropic),
        "google" => Some(ProviderKind::Google),
        "openai" => Some(ProviderKind::OpenAI),
        "deepseek" => Some(ProviderKind::Deepseek),
        "glm" => Some(ProviderKind::GLM),
        "kimi" => Some(ProviderKind::Kimi),
        _ => None,
    }
}

/// Reads a `KEY=VALUE` env file asynchronously and returns `(kind, api_key, model)`.
///
/// If the `.env` file is missing `PROVIDER`, `MODEL`, or a required `API_KEY`, this function
/// prompts the user interactively (via [`manual_config::load`]) and saves their answers to `path`.
pub async fn load() -> Option<(ProviderKind, Option<String>, String)> {
    // 1. Ask if we should use `.hanami/.env` or a custom path.
    let choice = Select::new(
        "Use the env file at .hanami/.env?",
        vec!["yes", "no (enter custom path)"],
    )
    .prompt()
    .ok()?;

    let path = if choice == "yes" {
        DEFAULT_ENV_PATH.to_string()
    } else {
        Text::new("Enter path to your .env file:")
            .prompt()
            .ok()?
            .trim()
            .to_string()
    };

    // 2. Make sure the file exists (creates an empty template if missing).
    if let Err(e) = create_env_file_if_missing(&path).await {
        println!("Error: could not create '{path}': {e}");
        return None;
    }

    // 3. Read the entire file into a `String` asynchronously with Tokio.
    let content = match tokio::fs::read_to_string(&path).await {
        Ok(text) => text,
        Err(e) => {
            println!("Error: could not read '{path}': {e}");
            return None;
        }
    };

    // 4. Parse `KEY=VALUE` lines.
    let mut provider: Option<ProviderKind> = None;
    let mut api_key: Option<String> = None;
    let mut model: Option<String> = None;

    for line in content.lines() {
        let line = line.trim();
        // Skip blank lines and `#` comments.
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // `split_once('=')` splits at the FIRST `=` only.
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        let key = key.trim();
        // Strip optional surrounding quotes: `MODEL="gemini-2.5-flash"` -> `gemini-2.5-flash`
        let value = value.trim().trim_matches('"').trim_matches('\'');

        match key {
            "PROVIDER" if !value.is_empty() => provider = parse_provider(value),
            "API_KEY" if !value.is_empty() => api_key = Some(value.to_string()),
            "MODEL" if !value.is_empty() => model = Some(value.to_string()),
            _ => {}
        }
    }

    // 5. Check if all required keys are present in the `.env` file.
    //    If any required key is missing, ask the user and save the answers to `.env`.
    let has_valid_config = match (&provider, &model) {
        (Some(kind), Some(_)) => !kind.needs_api_key() || api_key.is_some(),
        _ => false,
    };

    if has_valid_config {
        return Some((provider.unwrap(), api_key, model.unwrap()));
    }

    println!("'{path}' is missing configuration keys. Let's set it up and save to '{path}':");
    let (kind, api_key, model) = manual_config::load()?;

    let new_content = format_env_content(kind, api_key.as_deref(), &model);
    if let Err(e) = tokio::fs::write(&path, new_content).await {
        println!("Warning: could not save configuration to '{path}': {e}");
    } else {
        println!("Saved configuration to '{path}'.");
    }

    Some((kind, api_key, model))
}
