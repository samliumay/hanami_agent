//! Reads and saves provider API keys in `.hanami/.env` (or a custom `.env` file).
//!
//! - **Startup check (`ensure_initialized`)**: Checks if `.hanami/.env` exists to detect
//!   whether this is the first run. If missing, creates `.hanami/` and an empty `.hanami/.env`.
//! - **Provider selection (`load`)**: Reads the `.env` file and shows every provider in a menu
//!   along with its status (`[configured]` or `[not configured]`).
//! - **Auto-save missing keys**: If the user selects a provider that is `[not configured]`,
//!   Hanami prompts for that provider's API key and saves it into the `.env` file for future runs.

use crate::providers::ProviderKind;
use inquire::{Password, PasswordDisplayMode, Select, Text};
use std::collections::HashMap;
use std::path::Path;

/// Default location for the environment configuration file inside the workspace.
pub const DEFAULT_ENV_PATH: &str = ".hanami/.env";

/// Empty template written when `.hanami/.env` is created on the first run.
const EMPTY_ENV_TEMPLATE: &str = "\
# Hanami Agent API Keys
ANTHROPIC_API_KEY=
GOOGLE_API_KEY=
OPENAI_API_KEY=
DEEPSEEK_API_KEY=
GLM_API_KEY=
KIMI_API_KEY=
";

/// Maps each provider to its corresponding environment variable name in `.env`.
/// Returns `None` for Ollama because it runs locally without an API key.
fn env_key_for(kind: ProviderKind) -> Option<&'static str> {
    match kind {
        ProviderKind::Ollama => None,
        ProviderKind::Anthropic => Some("ANTHROPIC_API_KEY"),
        ProviderKind::Google => Some("GOOGLE_API_KEY"),
        ProviderKind::OpenAI => Some("OPENAI_API_KEY"),
        ProviderKind::Deepseek => Some("DEEPSEEK_API_KEY"),
        ProviderKind::GLM => Some("GLM_API_KEY"),
        ProviderKind::Kimi => Some("KIMI_API_KEY"),
    }
}

/// A menu item for `inquire::Select` that displays both the provider name and
/// whether its API key is already configured in the `.env` file.
#[derive(Clone, Copy)]
struct ProviderEntry {
    kind: ProviderKind,
    configured: bool,
}

impl std::fmt::Display for ProviderEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = if self.configured {
            "configured"
        } else {
            "not configured"
        };
        write!(f, "{} ({status})", self.kind)
    }
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

/// Parses non-empty `KEY=VALUE` pairs from `.env` text into a `HashMap`.
fn parse_env_map(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // `split_once('=')` splits at the FIRST `=` only so base64 keys with `=` stay intact.
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        let key = key.trim();
        let value = value.trim().trim_matches('"').trim_matches('\'');

        if !key.is_empty() && !value.is_empty() {
            map.insert(key.to_string(), value.to_string());
        }
    }

    map
}

/// Serializes the `HashMap` of keys back into `.env` format so all configured providers stay saved.
fn serialize_env_map(keys: &HashMap<String, String>) -> String {
    let mut out = String::from("# Hanami Agent API Keys\n");
    for kind in ProviderKind::ALL {
        if let Some(env_key) = env_key_for(kind) {
            let val = keys.get(env_key).map(String::as_str).unwrap_or("");
            out.push_str(&format!("{env_key}={val}\n"));
        }
    }
    out
}

/// Loads provider API keys from `.hanami/.env` (or a custom path), shows each provider's
/// configuration status in the selection menu, prompts & saves the key if an unconfigured
/// provider is chosen, and asks which model to use.
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

    // 3. Read the `.env` file and parse configured keys.
    let content = match tokio::fs::read_to_string(&path).await {
        Ok(text) => text,
        Err(e) => {
            println!("Error: could not read '{path}': {e}");
            return None;
        }
    };
    let mut keys = parse_env_map(&content);

    // 4. Build the provider menu showing `(configured)` or `(not configured)` on every entry.
    let entries: Vec<ProviderEntry> = ProviderKind::ALL
        .iter()
        .map(|&kind| {
            let configured = match env_key_for(kind) {
                None => true, // Ollama needs no key, so it is always ready.
                Some(env_key) => keys.contains_key(env_key),
            };
            ProviderEntry { kind, configured }
        })
        .collect();

    let chosen = Select::new("Which provider do you want to move with?", entries)
        .prompt()
        .ok()?;
    let kind = chosen.kind;

    // 5. Get the API key from `.env`, or ask for it and save it to `.env` if missing.
    let api_key = match env_key_for(kind) {
        None => None,
        Some(env_key) => {
            if let Some(existing_key) = keys.get(env_key) {
                Some(existing_key.clone())
            } else {
                let new_key = Password::new(&format!(
                    "'{kind}' is not configured in '{path}'. Please enter your {env_key}:"
                ))
                .with_display_mode(PasswordDisplayMode::Masked)
                .without_confirmation()
                .prompt()
                .ok()?;

                let new_key = new_key.trim().to_string();
                if new_key.is_empty() {
                    println!("Error: API key cannot be empty for '{kind}'.");
                    return None;
                }

                // Save the newly entered key into the map and write back to `.env`.
                keys.insert(env_key.to_string(), new_key.clone());
                let updated_env = serialize_env_map(&keys);
                if let Err(e) = tokio::fs::write(&path, updated_env).await {
                    println!("Warning: could not save key to '{path}': {e}");
                } else {
                    println!("Saved {env_key} to '{path}'.");
                }

                Some(new_key)
            }
        }
    };

    // 6. Ask which model to use for the selected provider.
    let model = Select::new("Which model do you want to use?", kind.models())
        .prompt()
        .ok()?;

    Some((kind, api_key, model.to_string()))
}
