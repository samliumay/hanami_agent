pub mod env_config;
pub mod manual_config;

use crate::providers::ProviderKind;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfigurationStyle {
    Env,
    Manual,
}

impl ConfigurationStyle {
    pub const ALL: [ConfigurationStyle; 2] = [Self::Env, Self::Manual];
}

impl std::fmt::Display for ConfigurationStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Env => "Env",
            Self::Manual => "Manual",
        };
        write!(f, "{name}")
    }
}

/// Returns `(kind, api_key, model)`, or `None` if the user cancelled or the `.env` file was invalid.
///
/// Marked `async` because the `Env` branch uses `tokio::fs` to create `.hanami/` and read `.env`.
pub async fn configure(
    style: ConfigurationStyle,
) -> Option<(ProviderKind, Option<String>, String)> {
    match style {
        ConfigurationStyle::Env => env_config::load().await,
        ConfigurationStyle::Manual => manual_config::load(),
    }
}
