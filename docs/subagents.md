# Subagent System

#hanami_agent #subagents #rust #async

Related files:
- [[../hanami_agent/src/agents.rs|src/agents.rs]]
- [[../hanami_agent/src/tools/spawn_agent.rs|src/tools/spawn_agent.rs]]
- [[../hanami_agent/src/tools.rs|src/tools.rs]]
- [[../hanami_agent/src/main.rs|src/main.rs]]

---

## Overview

The subagent system lets the main agent delegate focused tasks (like exploring files or making code edits) to an **isolated subagent** with its own conversation history, custom system prompt, and restricted toolset.

```text
main.rs
  └─► agents::run ("main", full history, all tools)
        └─► tools::execute ("create_subagent")
              └─► spawn_agent::run
                    └─► Box::pin(agents::run ("explorer" | "coder", fresh history, filtered tools))
```

---

## Modules & Responsibilities

### 1. `src/agents.rs`
Holds the subagent definitions and the shared agent execution loop:

- **`AgentDef`**: Struct describing a subagent preset:
  - `name`: Identifier passed to `create_subagent` (`"explorer"`, `"coder"`).
  - `description`: Shown to the parent model in the tool schema.
  - `system_prompt`: Replaces the parent's system prompt for the subagent's turns.
  - `tools`: Allowlist of tool names this subagent can call. `"create_subagent"` is excluded so subagents cannot spawn further subagents.
  - `max_rounds`: Maximum tool-call rounds before stopping.
- **`AGENTS` & `find_agent(name)`**: Static list of built-in `AgentDef`s and lookup helper.
- **`run(config, history, tools, label, max_rounds)`**:
  - Shared loop used by both `main.rs` (`label = "main"`) and subagents (`label = "explorer"` / `"coder"`).
  - Calls `providers::send()`. On `Reply::Text`, pushes `ChatMessage::Assistant` and returns the final answer. On `Reply::ToolCalls`, prints `[{label} -> tool]`, executes each tool via `tools::execute(call, config)`, records `ToolCalls` + `ToolResult`s, and continues.

### 2. `src/tools/spawn_agent.rs`
Implements the `"create_subagent"` tool:

- **`spec()`**: Dynamically builds the JSON Schema `enum` of valid agent names from `AGENTS`.
- **`run(args, parent_config)`**:
  1. Reads `"agent"` and `"user_prompt"` from `args`.
  2. Looks up the matching `AgentDef` via `agents::find_agent(agent_name)`.
  3. Clones `parent_config` (`ProviderConfig`) and replaces `system_prompt` with `agent_def.system_prompt`.
  4. Filters `tools::specs()` to only the tools listed in `agent_def.tools`.
  5. Creates a fresh `sub_history = vec![ChatMessage::User(user_prompt)]`.
  6. Runs `Box::pin(agents::run(...)).await` and returns the subagent's final text reply (or error message) back to the main agent as the tool output.

> [!note] Why `Box::pin` is used in `spawn_agent::run`
> Because `agents::run` calls `tools::execute`, which calls `spawn_agent::run`, which calls `agents::run`, Rust sees a recursive `async fn` cycle. Wrapping the inner `agents::run` future in `Box::pin(...)` puts it on the heap so the compiler knows the exact size of the `Future` at compile time.
