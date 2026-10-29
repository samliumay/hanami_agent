# <img src="./hanami-blossom.png" width="96" alt="Hanami Products blossom logo" /> Hanami Agent
https://github.com/samliumay/hanami_agent/pull/1/conflict?name=README.md&ancestor_oid=8123571886db1d4c3538fd7e3434949738584ee2&base_oid=99471b7ace9c4a2e23f8200a6cb4f2f7b197158f&head_oid=b91b2b50b4a554883f8285af5bd0f4a38e98835b
[![CI](https://github.com/samliumay/hanami_agent/actions/workflows/ci.yml/badge.svg)](https://github.com/samliumay/hanami_agent/actions/workflows/ci.yml)
![version](https://img.shields.io/badge/version-0.1.0-blue)

A terminal AI agent written from scratch in Rust. You chat with a model, and it can use tools on
your machine (read and write files, run shell commands) to get things done.

> **Done with curiosity and Rust.** This is a learning project. The code is full of notes and open
> questions, and it is not hardened. PRs are very welcome.

## Features

- Configure interactively in the terminal (`Manual`, with masked API keys) or load from a `.env` file (`Env`, defaulting to `.hanami/.env`).
- Multi-turn chat with tool calling (up to 10 tool rounds per message).
- Specialized **subagents** (`explorer` for read-only research, `coder` for making changes) with isolated context and toolsets.
- `@path/to/file` in a message attaches that file, with path autocomplete.
- `write_file`, `edit_file`, `run_command`, and reading files outside the workspace ask for your approval before they run.
- Minimal by design: no web search tool. The model can use `curl` through `run_command` when it
  needs the web.

## Providers

| Provider                          | Status          |
| --------------------------------- | --------------- |
| Google Gemini                     | ✅ chat + tools |
| Anthropic Claude                  | ✅ chat + tools |
| OpenAI (Responses API)            | ✅ chat + tools |
| Ollama (local, `localhost:11434`) | ✅ chat + tools |
| DeepSeek                          | ✅ chat + tools |
| GLM (Z.ai)                        | ✅ chat + tools |
| Moonshot Kimi                     | ✅ chat + tools |

## Tools

| Tool               | What it does                                                          |
| ------------------ | --------------------------------------------------------------------- |
| `get_current_time` | Returns the local/UTC date and time                                   |
| `read_file`        | Reads a text file (free inside the workspace, asks first outside)     |
| `write_file`       | Creates or overwrites a file (asks first)                             |
| `edit_file`        | Replaces old text with new text in an existing file (asks first)      |
| `run_command`      | Runs a shell command, 30 s timeout (asks first)                       |
| `create_subagent`  | Spawns an isolated subagent (`explorer` or `coder`) for a subtask     |

## Quick start

Needs Rust 1.85 or newer ([rustup](https://rustup.rs)).

```bash
git clone https://github.com/samliumay/hanami_agent.git
cd hanami_agent/hanami_agent
cargo run --release
```

At startup, Hanami checks whether `.hanami/.env` exists (creating `.hanami/` and an empty `.hanami/.env` on the first run), then asks how you want to configure the session:

- **`Env`**: Reads `.hanami/.env` (or a custom `.env` path) and asks **which provider you want to move with**, showing **`(configured)`** or **`(not configured)`** next to every provider entry. If you choose a provider that is `(not configured)`, Hanami prompts for its API key and saves it into the `.env` file for future runs.
- **`Manual`**: Prompts for the provider, API key (masked, never stored on disk), and model for this session only.

Lines starting with `[main -> tool]` (or `[explorer -> tool]` / `[coder -> tool]`) show the tools the agent runs. Type `/exit` to quit.

See [`docs/README.md`](docs/README.md) for an overview of the codebase and module architecture.

`run_command` uses `sh`, so it works on Linux and macOS but not on Windows yet. Even with the
approval prompts, it's best to run Hanami Agent in a folder you don't mind changing.

Requests to providers time out after 120 s. GLM uses the standard Z.ai endpoint; GLM Coding Plan
keys need the `api/coding/paas/v4` URL in `src/providers/glm.rs`.

## Branches

- `main` is always the usable version. Clone this one.
- `dev` is where new work is tested before it is merged into `main`.

## Roadmap

- [x] Basic function calls and the base of the project.
- [x] Chat loop, tool calling, approval prompts, `@file` mentions
- [x] Gemini, Claude, OpenAI, Ollama, DeepSeek, GLM and Kimi providers
- [x] `edit_file` tool
- [x] Configuration & API keys from `.env` file (`.hanami/.env`)
- [x] Subagent system (`create_subagent` with `explorer` and `coder`)
- [ ] Dynamic file-based subagent definitions in `.hanami/agents/`
- [ ] Windows support, streaming responses, better tests
