# Configuration System

#hanami_agent #configuration #tokio

Related files:
- [[../hanami_agent/src/configuration.rs|src/configuration.rs]]
- [[../hanami_agent/src/configuration/env_config.rs|src/configuration/env_config.rs]]
- [[../hanami_agent/src/configuration/manual_config.rs|src/configuration/manual_config.rs]]

---

## Overview

At startup, `main.rs` asks whether to configure Hanami from an `.env` file (`ConfigurationStyle::Env`) or interactively (`ConfigurationStyle::Manual`).

Both paths return the same tuple:

```rust
Option<(ProviderKind, Option<String>, String)>
//      ^^^^^^^^^^^^  ^^^^^^^^^^^^^^  ^^^^^^
//      provider      api_key         model name
```

Returning `None` (e.g., user pressed `Ctrl+C`/`Esc` or `.env` was missing required keys) causes `main.rs` to exit cleanly.

---

## Modules

### 1. `configuration.rs`
Defines `ConfigurationStyle` (`Env` | `Manual`) and dispatches via `pub async fn configure(style)`.

### 2. `configuration/env_config.rs`
Handles first-run initialization and per-provider API keys in `.hanami/.env` (or a custom path):
1. **First-run check at startup (`ensure_initialized()`)**: Called first in `main.rs`. Checks `tokio::fs::try_exists(".hanami/.env").await`.
   - If `.hanami/.env` does **not** exist (first run), creates `.hanami/` and writes an empty `.hanami/.env` template with keys for each provider (`ANTHROPIC_API_KEY=`, `GOOGLE_API_KEY=`, etc.).
   - If `.hanami/.env` already exists, does nothing.
2. **Provider menu with status (`load()`)**: Reads the `.env` file into a `HashMap` and asks which provider you want to move with, displaying **`(configured)`** or **`(not configured)`** next to every provider entry (`ProviderEntry`).
3. **Prompt & save on missing key**: If you pick a provider that is `(not configured)`, it asks for that provider's API key (masked) and writes the updated key map back to the `.env` file so it shows up as `(configured)` on future runs.

#### Example `.hanami/.env`
```env
# Hanami Agent API Keys
ANTHROPIC_API_KEY=
GOOGLE_API_KEY=your_api_key_here
OPENAI_API_KEY=
DEEPSEEK_API_KEY=
GLM_API_KEY=
KIMI_API_KEY=
```

### 3. `configuration/manual_config.rs`
Prompts the user interactively in the terminal using `inquire`:
1. `Select` for `ProviderKind`.
2. `Password` (masked) for the API key if `kind.needs_api_key()` is `true`.
3. `Select` for the model from `kind.models()`.
