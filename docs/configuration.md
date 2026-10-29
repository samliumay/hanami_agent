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
Handles first-run initialization and reading/saving `.hanami/.env` (or a custom path):
1. **First-run check at startup (`ensure_initialized()`)**: Called first in `main.rs`. It checks `tokio::fs::try_exists(".hanami/.env").await`.
   - If `.hanami/.env` does **not** exist (first run), it creates `.hanami/` (`tokio::fs::create_dir_all`) and creates an empty `.hanami/.env` template (`tokio::fs::write`).
   - If `.hanami/.env` already exists, it does nothing.
2. **Loading (`load()`)**: Ensures the selected `.env` file exists, reads it with `tokio::fs::read_to_string(&path).await`, and parses `KEY=VALUE` lines using `line.split_once('=')`.
3. **Interactive fallback & save**: If `PROVIDER`, `MODEL`, or (when required) `API_KEY` are missing in the `.env` file, it prompts the user via `manual_config::load()` and saves their answers into the `.env` file with `tokio::fs::write` so future runs load automatically.

#### Example `.hanami/.env`
```env
PROVIDER=google
MODEL=gemini-2.5-flash
API_KEY=your_api_key_here
```

### 3. `configuration/manual_config.rs`
Prompts the user interactively in the terminal using `inquire`:
1. `Select` for `ProviderKind`.
2. `Password` (masked) for the API key if `kind.needs_api_key()` is `true`.
3. `Select` for the model from `kind.models()`.
