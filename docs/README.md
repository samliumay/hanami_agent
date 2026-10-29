# Hanami Agent — Code & Module Documentation

#hanami_agent #architecture #rust

Simple overview of the crate structure in `hanami_agent/src/` and how each module fits together.

---

## Module Map

```text
hanami_agent/src/
├── main.rs                    # Entry point: config setup + interactive REPL loop
├── agents.rs                  # AgentDef presets ("explorer", "coder") + shared agents::run loop
├── configuration.rs           # ConfigurationStyle enum (Env / Manual) + configure() dispatcher
├── configuration/
│   ├── env_config.rs          # Ensures .hanami/ exists and parses KEY=VALUE from .hanami/.env
│   └── manual_config.rs       # Interactive prompts for provider, API key, and model
├── providers.rs               # ProviderKind, ProviderConfig, ChatMessage, Reply + send() dispatcher
├── providers/
│   ├── anthropic.rs           # Claude Messages API
│   ├── deepseek.rs            # DeepSeek API (with reasoning_content)
│   ├── glm.rs                 # Z.ai GLM API
│   ├── google.rs              # Google Gemini API
│   ├── kimi.rs                # Moonshot Kimi API
│   ├── ollama.rs              # Local Ollama API
│   └── openai.rs              # OpenAI Responses API
├── tools.rs                   # ToolSpec, ToolCall, approval prompts, and tools::execute()
├── tools/
│   ├── edit_file.rs           # Search-and-replace file editing
│   ├── read_file.rs           # Safe file reading (free inside workspace, prompts outside)
│   ├── run_command.rs         # Shell command execution with 30s timeout
│   ├── spawn_agent.rs         # Spawns isolated subagents ("create_subagent" tool)
│   ├── time.rs                # Returns current local/UTC time
│   └── write_file.rs          # Creates or overwrites files
└── helpers.rs                 # @file mention expansion, autocomplete, terminal sanitization
```

---

## Topic Guides

- [[configuration|Configuration System (.hanami/.env & Manual Setup)]]
- [[subagents|Subagent System (agents.rs & spawn_agent.rs)]]
- [[../questions_and_answers|Rust Study Notes (Questions & Answers)]]
- [[../fixed_bugs|Fixed Bugs Log]]

---

## End-to-End Request Flow

1. **Startup (`main.rs`)**:
   - Asks the user to choose `Env` or `Manual` via `configuration::configure()`.
   - Builds `ProviderConfig` and loads all `ToolSpec`s via `tools::specs()`.
2. **User Turn (`main.rs` -> `agents::run`)**:
   - Expands any `@file` mentions in the user's message and appends `ChatMessage::User` to `history`.
   - Calls `agents::run(&provider_details, &mut history, &tool_specs, "main", 10)`.
3. **Agent Loop (`agents.rs`)**:
   - Calls `providers::send()` for the chosen provider.
   - If the model returns `Reply::Text`, pushes `ChatMessage::Assistant` and returns the text.
   - If the model returns `Reply::ToolCalls`, runs each tool via `tools::execute(call, config)` (asking user approval when required), appends `ToolCalls` + `ToolResult`s to `history`, and loops for the next round.
