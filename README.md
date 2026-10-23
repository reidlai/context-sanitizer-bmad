# context-sanitizer-bmad

[![Sponsor on GitHub](https://img.shields.io/badge/Sponsor-reidlai-ea4aaa?logo=github-sponsors)](https://github.com/sponsors/reidlai)

This repository houses the BMad planning, design artifacts, agent personas, and development specifications for `context-sanitizer`.

## Repository & Submodule Structure

This project uses a two-repo submodule layout:

- **Root Repository (`context-sanitizer-bmad`)**:
  - `_bmad-output/`: Planning documents, architecture decision records (ADRs), user stories, PRD, and implementation artifacts.
  - `.agent/`: BMad agent definitions, skills, and elicitation workflows.
  - Submodule pointer to `context-sanitizer/`.

- **Core Submodule (`context-sanitizer/`)**:
  - Located at `context-sanitizer/` (git submodule tracking active feature branches).
  - Contains all pure-Rust production code, crates, tests, and CI workflows:
    - [`crates/context-sanitizer`](file:///c:/Users/reidl/GitLocal/context-sanitizer-bmad/context-sanitizer/crates/context-sanitizer): Core library providing Tier 1 deterministic secret redaction, operational PII shadow vaulting, Tier 2 syntax-safe statistical context compression, and OpenAI wire transformers.
    - [`crates/context-sanitizer-cli`](file:///c:/Users/reidl/GitLocal/context-sanitizer-bmad/context-sanitizer/crates/context-sanitizer-cli): Standalone `context-sanitizer` binary providing the `xray` latency dashboard and automated benchmarking harness.
    - [`crates/context-sanitizer-py`](file:///c:/Users/reidl/GitLocal/context-sanitizer-bmad/context-sanitizer/crates/context-sanitizer-py): Zero-C Python wheel & turnkey agent middleware for LangChain, CrewAI, AutoGen, and FastAPI.
    - [`wasm/`](file:///c:/Users/reidl/GitLocal/context-sanitizer-bmad/context-sanitizer/wasm): Universal WebAssembly / WASI bindings.
    - `.github/workflows/ci.yml`: 6-stage DevSecOps CI pipeline.

---

## The 3 Workspace Crates & How to Use Them

### 1. `context-sanitizer` (Pure-Rust Core Library)

Zero-C, memory-safe library for LLM context sanitization, PII Shadow Vaulting, and statistical token compression.

#### Installation
```bash
cargo add context-sanitizer
```

#### Usage Example
```rust
use context_sanitizer::{compress_chat_json, sanitize_str_with_vault, VaultHandle};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize a session-scoped cryptographic VaultHandle
    let vault = VaultHandle::new();
    vault.allow_destination("db_lookup");

    // 2. Deterministic secret redaction + operational PII shadow tokenization
    let dirty_prompt = "Admin key: ghp_123456789012345678901234567890123456. Contact alice@acme.com.";
    let safe_prompt = sanitize_str_with_vault(dirty_prompt, &vault);
    println!("Sanitized:\n{}", safe_prompt);
    // => "Admin key: [REDACTED_CREDENTIAL]. Contact <SHADOW:EMAIL_8f3a2c>."

    // 3. Syntax-safe statistical context compression (OpenAI Chat JSON)
    let chat_payload = r#"{
        "model": "gpt-4o",
        "messages": [
            {"role": "user", "content": "Please query user profile for alice@acme.com"}
        ]
    }"#;
    let (compressed_json, stats) = compress_chat_json(chat_payload, &vault, 0.40)?;
    println!("\nCompressed JSON:\n{}", compressed_json);
    println!("Saved {:.1}% tokens", (1.0 - stats.ratio) * 100.0);

    // 4. Local tool call rehydration (authorized destination only)
    let tool_call = r#"{"tool": "db_lookup", "email": "<SHADOW:EMAIL_8f3a2c>"}"#;
    let rehydrated_args = vault.rehydrate(tool_call, "db_lookup")?;
    println!("\nRehydrated Arguments:\n{}", rehydrated_args);

    Ok(())
}
```

---

### 2. `context-sanitizer-cli` (Context X-Ray Diagnostic & Benchmark CLI)

Terminal diagnostic tool for visualizing token pruning, shadow-vaulted PII, redacted credentials, and nanosecond latency SLAs.

#### Installation
```bash
# Install binary from crates.io
cargo install context-sanitizer-cli

# Or run directly inside the workspace
cd context-sanitizer
cargo run --bin context-sanitizer -- xray --help
```

#### Usage Commands
```bash
# 1. Inspect and visualize a prompt payload
context-sanitizer xray prompt.json --rate 0.35

# 2. Run the automated throughput, compression, and adversarial anti-DoS benchmark suite
context-sanitizer xray --bench
```

#### Visualizer Preview
```text
╭─ Context X-Ray: prompt.json ─────────────────────────────────────────────────╮
│ Tier 1 (L0 Guard)    : 1 PII Vaulted | 1 Root Key Redacted | 8 syntax locks  │
│ Tier 2 (L0.5 Rust)   : Pruned 1,240 low-entropy & schema bloat tokens         │
│ Tier 3 (L1 System 1) : Threat Score 0.01 (SAFE)                              │
├──────────────────────────────────────────────────────────────────────────────┤
│ User: Please check status for <SHADOW:EMAIL_8f3a2c>                          │
│ API_KEY: [REDACTED_CREDENTIAL]                                               │
├──────────────────────────────────────────────────────────────────────────────┤
│ Tokens : 2,400 ──► 840 (65.0% saved | Target: 0.35)                          │
│ Savings: ~$0.0234 / 1k calls (GPT-4o)  |  Latency: 384,000 ns (L0+L0.5)       │
╰──────────────────────────────────────────────────────────────────────────────╯
```

---

### 3. `context-sanitizer-py` (Python Wheel & Agent Framework Middleware)

High-performance Python extension built with PyO3 and Maturin, releasing the GIL during processing.

#### Installation
```bash
# Install precompiled wheel from PyPI
pip install context-sanitizer-py

# Or compile and install locally in an active virtualenv
cd context-sanitizer/crates/context-sanitizer-py
uv run --with maturin maturin develop
```

#### Usage Example
```python
import context_sanitizer as cs

# 1. Sanitize text or chat messages with automatic vaulting
dirty_prompt = """
My GitHub token is ghp_123456789012345678901234567890123456.
Please lookup account details for alice@acme.com and server 192.168.1.1.
"""

sanitized_text, vault = cs.sanitize(
    dirty_prompt,
    rate=0.40,
    whitelist=["db_lookup", "network_ping"],
)

print("--- Safe Prompt for Cloud LLM ---")
print(sanitized_text)
# Root secret replaced with [REDACTED_CREDENTIAL]
# alice@acme.com replaced with <SHADOW:EMAIL_8f3a2c>
# 192.168.1.1 replaced with <SHADOW:IP_1a2b3c>

# 2. Rehydrate shadow tokens locally when executing agent tool calls
llm_emitted_call = '{"tool": "db_lookup", "email": "<SHADOW:EMAIL_8f3a2c>"}'
local_args = vault.rehydrate(llm_emitted_call, destination="db_lookup")
print("\n--- Rehydrated Local Tool Arguments ---")
print(local_args)

# 3. Turnkey Agent Framework Middleware
from context_sanitizer.integrations import LangChainSanitizer, CrewAISanitizer
```

---

## Quick Start: Developer Setup

### Cloning with Submodules

```bash
git clone --recurse-submodules https://github.com/reidlai/context-sanitizer-bmad.git
cd context-sanitizer-bmad
```

If already cloned:

```bash
git submodule update --init --recursive
```

### Building and Testing Workspace

All Rust development is executed inside the submodule:

```bash
cd context-sanitizer

# Build all workspace crates (core, cli, py, wasm)
cargo build

# Run entire test suite
cargo test --workspace

# Run release compression tests
cargo test --release --test e2e_tier2_compression

# Run CLI benchmark harness
cargo run --bin context-sanitizer -- xray --bench
```
