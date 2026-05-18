# claw-code

> A multi-provider fork of [Claude Code](https://github.com/anthropics/claude-code) with planned Python embedding via PyO3 and integration with the harnais quality governance framework.

[![Rust](https://img.shields.io/badge/rust-1.78%2B-orange)](https://www.rust-lang.org)
[![Python](https://img.shields.io/badge/python-3.11%2B-blue)](https://www.python.org)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)

---

## Overview

**claw-code** is a personal R&D fork of [Anthropic's Claude Code](https://github.com/anthropics/claude-code). It extends the original agent CLI with multi-provider routing, planned Python embedding, and integration with a quality governance framework called **harnais**.

This is **not a competing product** with Claude Code. It is a downstream experiment focused on three specific use cases:

1. **Local-first inference** for personal workflows where data should not leave the machine
2. **Multi-provider abstraction** across Anthropic, Ollama, OpenAI-compatible APIs, and others
3. **Programmatic governance** via pre-commit gates, knowledge accumulation, and autonomous mission orchestration

Upstream improvements from `anthropics/claude-code` are tracked and merged regularly. Contributions back upstream are encouraged where they make sense.

---

## What's different from upstream

| Feature | Upstream `claude-code` | This fork `claw-code` |
|---------|------------------------|------------------------|
| LLM providers | Anthropic | Anthropic · Ollama · OpenAI-compatible · DeepSeek · xAI |
| Streaming SSE | ✅ | ✅ |
| MCP servers | ✅ | ✅ |
| Sessions | ✅ | ✅ |
| Skills system | ✅ | ✅ + harnais extensions |
| Worktree integration | Basic | Integrated with autonomous missions |
| Python embedding | — | PyO3 0.21 (in progress, v14) |
| Quality governance | — | Pre-commit gates via harnais |
| Offline mode | Limited | Full standalone (planned, v14) |
| Knowledge accumulator | — | JSONL-based, validated insights |

---

## Status

| Milestone | Status | Notes |
|-----------|--------|-------|
| Multi-provider routing | ✅ Shipped | Anthropic, Ollama, OpenAI-compat, DeepSeek, xAI |
| MCP integration | ✅ Shipped | 6 modules |
| Skills system | ✅ Shipped | SKILL.md compatible |
| Phase 0 cleanup | ✅ Shipped | Provider imports fixed, security advisories resolved |
| Phase B (v14 foundation) | 🔄 In progress | PyO3 bridge crate (`harnais-ffi`) |
| Phase C (Symphony bridge) | 📋 Planned | Python SDK, app wiring, tenant routing |

---

## Architecture (high-level)

```
┌──────────────────────────────────────────────────────────┐
│ claw-code binary (Rust workspace, 11 crates)             │
├──────────────────────────────────────────────────────────┤
│ CLI (clap 4) — sessions, agents, skills, missions        │
├──────────────────────────────────────────────────────────┤
│ Provider abstraction (AIProvider trait)                  │
│  ├─ Anthropic (streaming SSE)                            │
│  ├─ Ollama (local)                                       │
│  ├─ OpenAI-compatible (xAI Grok, etc.)                   │
│  └─ DeepSeek                                             │
├──────────────────────────────────────────────────────────┤
│ MCP client (Model Context Protocol)                      │
├──────────────────────────────────────────────────────────┤
│ Telemetry · Sessions · Storage · Skills runtime          │
└──────────────────────────────────────────────────────────┘
                          ▼
         (planned v14) PyO3 bridge to Python tooling
         ├─ sentence-transformers (LaBSE embeddings)
         ├─ Ollama HTTP client
         ├─ Context Broker (PostgreSQL + pgvector)
         └─ Knowledge Accumulator (JSONL)
```

---

## Installation

### Prerequisites

- **Rust**: 1.78 or later (recommended: latest stable via `rustup`)
- **Python**: 3.11 or later (for v14 PyO3 features)
- **Ollama**: optional, for local inference

### Build from source

```bash
git clone https://github.com/TaZLeLuTiN/claw-code.git
cd claw-code/rust
cargo build --release

# The binary is at target/release/claw-code
# Add it to your PATH or symlink it:
sudo ln -s "$(pwd)/target/release/claw" /usr/local/bin/claw
```

### Verify

```bash
claw --version
```

---

## Quick start

```bash
# Authenticate with Anthropic
claw auth login

# Start an interactive session
claw

# Or run a one-shot command
claw "Refactor this function to use async/await"

# Use a different provider (e.g. local Ollama)
claw --provider ollama --model gemma3:4b "Explain this code"

# List available skills
claw skill list
```

See [`docs/`](docs/) for detailed documentation.

---

## Provider configuration

Providers are configured in `~/.config/claw/providers.yaml`:

```yaml
providers:
  anthropic:
    api_key_env: ANTHROPIC_API_KEY
    default_model: claude-sonnet-4

  ollama:
    base_url: http://localhost:11434
    default_model: gemma3:4b

  openai_compat:
    base_url: https://api.x.ai/v1
    api_key_env: XAI_API_KEY
    default_model: grok-2

  deepseek:
    api_key_env: DEEPSEEK_API_KEY
    default_model: deepseek-chat
```

---

## Roadmap

### v14 — The PyO3 bridge milestone

The current major effort, focused on six themes:

1. **Foundation** — Rust ↔ Python bridge via PyO3 0.21
2. **Standalone offline-first** — full operation without network connectivity
3. **Skills system** — 30 production-ready skills compatible with the Anthropic SKILL.md standard
4. **Killer features** — 20 differentiators (multi-agent routing, autonomous missions, health-aware orchestration, etc.)
5. **Symphony bridge** — integration with the harnais v14 orchestration framework
6. **Multi-architecture distribution** — macOS arm64, Linux x64/arm64

Detailed planning lives in the private `harnais` repository. Public roadmap items are tracked as GitHub issues.

### Beyond v14

- LoRA fine-tuning pipeline integration
- Multi-machine federation via OcuLink
- Mobile companion (Tauri 2)
- Investor demo mode

---

## Project structure

```
claw-code/
├── rust/                    # Rust workspace (11 crates)
│   ├── crates/
│   │   ├── api/             # Streaming SSE, OAuth, raw HTTP
│   │   ├── runtime/         # Provider trait + implementations
│   │   ├── rusty-claude-cli # Main CLI binary
│   │   ├── commands/        # Built-in commands
│   │   ├── plugins/         # Plugin runtime
│   │   ├── telemetry/       # Observability
│   │   ├── tools/           # Built-in tools
│   │   ├── gui/             # Optional GUI
│   │   ├── compat-harness/  # Compatibility shims
│   │   ├── mock-anthropic-service/  # Test mocks
│   │   └── ffi/             # (v14) PyO3 bindings — coming soon
│   └── target/              # Build artifacts (gitignored)
├── docs/                    # Public documentation
└── README.md
```

---

## Credits

Forked from [Anthropic's Claude Code](https://github.com/anthropics/claude-code). Original work © Anthropic, PBC. Many thanks to the Anthropic team for releasing such a capable foundation as open source.

Modifications and additions © Mike Burini ([@TaZLeLuTiN](https://github.com/TaZLeLuTiN)).

---

## License

This fork inherits the [MIT License](LICENSE) of the upstream project. See the [`LICENSE`](LICENSE) file for the full text.

---

## Contributing

This is a personal R&D fork primarily driven by my own use cases. Issues are welcome for discussion, especially:

- Bug reports affecting multi-provider behavior
- Compatibility issues with upstream `claude-code` features
- Suggestions for the v14 roadmap

Pull requests are also welcome, but please open an issue first to discuss the change.

For commercial inquiries or integration partnerships, reach out directly.

---

## Disclaimer

This is **not an official Anthropic product**. Use at your own discretion. For production workloads relying on the official Anthropic API, the upstream [`claude-code`](https://github.com/anthropics/claude-code) is recommended.
