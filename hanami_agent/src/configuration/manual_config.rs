use crate::providers::ProviderKind;
use inquire::{Password, PasswordDisplayMode, Select};

/// Asks the user for provider, API key and model.
/// Returns None when the user cancels a prompt (Ctrl+C / Esc).
pub fn load() -> Option<(ProviderKind, Option<String>, String)> {
    let kind = Select::new(
        "Which provider are you going with?",
        ProviderKind::ALL.to_vec(),
    )
    .prompt()
    .ok()?;

    let api_key = if kind.needs_api_key() {
        let key = Password::new("Please enter your API key:")
            .with_display_mode(PasswordDisplayMode::Masked)
            .without_confirmation()
            .prompt()
            .ok()?;
        let key = key.trim();
        if key.is_empty() {
            None
        } else {
            Some(key.to_string())
        }
    } else {
        None
    };

    let model = Select::new("Which model do you want to use?", kind.models())
        .prompt()
        .ok()?;

    // Convert `&'static str` from the menu into an owned `String` so both
    // `manual_config::load` and `env_config::load` return the exact same type.
    Some((kind, api_key, model.to_string()))
}
