# context-sanitizer

**context-sanitizer** is a blazingly fast, local Rust proxy and library that intercepts LLM API traffic to act as a **System One semantic firewall** and token compressor.

Built for modern Agentic AI and Copilot workflows, it solves the "context rot" problem by trimming stale history and massive tool schemas, while ensuring your AWS keys, GitHub tokens, and proprietary semantic data never leak to third-party LLM providers. It offers a pure Rust core with seamless Python bindings via `PyO3`.

## The 3-Tier Hybrid Architecture

Built specifically to sit at the boundary of **Copilot UIs (including CopilotKit and OpenAI-compatible APIs)** and **Agentic AI frameworks**, `context-sanitizer` executes a **3-Tier Speculative Cascade** for sub-millisecond speed out-of-the-box and deep neural accuracy when enabled:

```text
Incoming CopilotUI / OpenAI API / Agent Payload
   │
   ▼
┌───────────────────────────────────────────────────────────────────────────┐
│ Tier 1 (L0): Pure-Rust Deterministic & Lexical Guard (< 0.5 ms | Always)  │
│  • aho-corasick SIMD secret & PII detection (Root Keys -> [REDACTED_*])   │
│  • Session-Scoped Shadow Vault (<SHADOW:EMAIL_8f3a2c> + .rehydrate() API) │
│  • Pure-Rust lexical/JSON scanner (logos + serde_json) syntax token lock  │
└───────────────────────────────────────────────────────────────────────────┘
   │
   ▼
┌───────────────────────────────────────────────────────────────────────────┐
│ Tier 2 (L0.5): Pure-Rust Smart Statistical Pruning (< 2 ms | 0 MB Weights)│
│  • Zero-weight information-theoretic pruning (tiktoken-rs + BM25/TF-IDF)  │
│  • Tail-query auto-anchoring, JSON schema minification & history dedup    │
└───────────────────────────────────────────────────────────────────────────┘
   │
   ▼
┌───────────────────────────────────────────────────────────────────────────┐
│ Tier 3 (L1): Configurable Trait-Based System One ML (--features ml)       │
│  • Pluggable traits (SemanticGuard + TokenCompressor) & HF model_id       │
│  • Stage 3A: Zero-Shot NLI / Prompt Guard circuit breaker (short-circuits)│
│  • Stage 3B: LLMLingua-2 BERT token classifier (runs on allowed payloads) │
└───────────────────────────────────────────────────────────────────────────┘
```

1. **The Configurable System One Gatekeeper (`SemanticGuard` Trait):** Uses blazing-fast, open-source System One encoder models (defaulting to *DeBERTa-v3 Zero-Shot NLI*, swappable via `guard_model_id` to *ModernBERT*, *Llama-Prompt-Guard*, or custom ONNX/Candle models) to block malicious code injections and proprietary semantic leaks in a single forward pass—short-circuiting before compression runs on blocked threats.
2. **The Session-Scoped "Shadow Vault" + Split Secret Policy:** Operational PII (emails, IPs, phones) is swapped with session-scoped cryptographic shadow tokens (e.g., `<SHADOW:EMAIL_8f3a2c>`) bound to a `VaultHandle` with a `.rehydrate()` API so agent tool calls keep working, while root credentials (AWS keys, GitHub PATs) default to one-way `[REDACTED_*]` masks so indirect prompt injections can never exfiltrate keys.
3. **The 100% Pure-Rust & ML Compression Engine (`TokenCompressor` Trait):** Leverages zero-C pure-Rust lexical syntax locking (`logos` + `serde_json`) and statistical pruning by default (0 MB model download required), plus optional neural token classification (*LLMLingua-2*, swappable via `compression_model_id`) to shrink payloads token-by-token without ever breaking code or JSON syntax.

## Research Foundation

This project implements state-of-the-art token pruning and semantic architectures based on recent publicly available research in context engineering:

* **System One Inference for Bounded Semantic Reasoning**: Shifts away from sequential token generation, allowing non-autoregressive, encoder-only architectures (like Microsoft's DeBERTa-v3 and Meta's Prompt Guard) to compute text semantics and detect indirect prompt/code injections in a single pass for near-zero-latency safety checks.
* **LLMLingua-2: Data Distillation for Efficient and Faithful Task-Agnostic Prompt Compression (ACL 2024)**: Microsoft's research formulates prompt compression as a token classification problem ($p(\text{keep})$). Using a bidirectional encoder, it achieves 3x-6x faster compression and accelerates end-to-end latency by 1.6x-2.9x while preserving downstream accuracy.
* **Selective Context & Query-Aware Submodular Pruning (RECOMP / Self-Information)**: Grounds our zero-weight Pure-Rust Tier 2 engine in information theory ($I(x) = -\log P(x)$) and query-aware BM25/submodular chunk selection to prune low-entropy filler tokens without requiring GPU weights.
* **Prompt Compression for Large Language Models: A Survey (2024)**: Establishes the academic taxonomy of prompt compression, dividing the field into "hard" compression (discrete token removal/pruning) and "soft" compression (trainable continuous vectors).

## Development Methodology

This project is engineered using **Spec-Driven Development (SDD)** guided by the **BMAD Method**.

Rather than ad-hoc implementation, all architecture and capabilities follow a disciplined, specification-first engineering lifecycle:

* **Specification-First Architecture**: Interface contracts, C-ABI boundaries (`PyO3`), PII/secret regex invariants, and System One guardrails are strictly formalized before code implementation.
* **Bounded Multi-Agent Execution**: Complex pipelines (such as hybrid lexical pruning vs. neural classification) are decomposed into discrete, bounded tasks with explicit validation criteria.
* **Deterministic Verification**: Specifications directly drive automated test suites across both the Rust workspace and Python bindings to guarantee zero secret leakage, predictable compression ratios, and cross-language memory safety.

## Features

* **System One Semantic Firewall & Code-Injection Gatekeeper**: Intercepts malicious code injections, jailbreaks, and proprietary semantic leaks entering through Copilot UIs (CopilotKit, OpenAI API `/v1/chat/completions`) before they reach your agentic framework.
* **Pluggable Model Traits (`SemanticGuard` & `TokenCompressor`)**: Ships with *DeBERTa-v3 Zero-Shot NLI* and *LLMLingua-2 BERT* defaults, while allowing users to swap in any Hugging Face or local checkpoint (`guard_model_id` / `compression_model_id`, e.g., *ModernBERT*, *Llama-Prompt-Guard*, or custom ONNX/Candle models) or implement the Rust traits directly.
* **Session-Scoped "Shadow Vault" + Split Secret Policy**: Replaces operational PII (emails, phone numbers, IPs, customer IDs) with session-scoped cryptographic tokens (`<SHADOW:EMAIL_8f3a2c>`) that can be rehydrated locally via `.rehydrate()` for tool calls, while root credentials (AWS keys, GitHub PATs, private keys) default to one-way `[REDACTED_*]` masks so indirect prompt injections can never exfiltrate keys.
* **3-Tier Hard Token Compression (100% Pure-Rust & Syntax-Safe)**: Combines a zero-C pure-Rust lexical/JSON scanner (`logos` + `serde_json`) for syntax token locking with Pure-Rust statistical pruning (Tier 2: `tiktoken-rs` + BM25/self-information with tail-query auto-anchoring) and *LLMLingua-2* neural token classification (Tier 3).
* **Context X-Ray CLI & Benchmark Visualizer (`context-sanitizer xray`)**: Built-in terminal diagnostic tool (`context-sanitizer-cli`) that visualizes pruned tokens, Shadow-Vaulted PII, one-way redacted secrets, System One threat scores, dollar savings, and microsecond latency.
* **Zero-Leak SIMD Regex Security**: Sub-millisecond interception of hardcoded secrets and PII using the `aho-corasick` SIMD multi-pattern search engine.
* **Modular 3-Crate Cargo Workspace**: Clean separation across `context-sanitizer` (core Rust library), `context-sanitizer-cli` (`xray` binary), and `context-sanitizer-py` (`PyO3` Python bindings).

## Context X-Ray CLI (`context-sanitizer xray`)

Inspect, benchmark, and visualize exactly how `context-sanitizer` protects and compresses your prompts in the terminal:

```bash
cargo install context-sanitizer-cli
context-sanitizer xray payload.json --rate 0.35
```

```text
╭─ Context X-Ray: payload.json ────────────────────────────────────────────────╮
│ Tier 1 (L0 Guard)    : 2 PII Vaulted | 1 Root Key Redacted | 14 syntax locks │
│ Tier 2 (L0.5 Rust)   : Pruned 1,420 low-entropy & schema bloat tokens        │
│ Tier 3 (L1 System 1) : Threat Score 0.03 (SAFE) | Route: Code+Chat           │
├──────────────────────────────────────────────────────────────────────────────┤
│ User: Please check login failure for <SHADOW:EMAIL_8f3a2c> on <SHADOW:IP_9b1e>│
│ Code: AWS_ACCESS_KEY_ID=[REDACTED_AWS_KEY]                                   │
│ function execute_deployment() {                                              │
│   [-// This is a very long comment that explains nothing-]                   │
│   return true;                                                               │
│ }                                                                            │
├──────────────────────────────────────────────────────────────────────────────┤
│ Tokens : 3,200 ──► 1,088 (66.0% saved | Target: 0.35)                        │
│ Savings: ~$0.0318 / 1k calls (GPT-4o)  |  Latency: 412 µs (L0+L0.5)          │
╰──────────────────────────────────────────────────────────────────────────────╯
```

## Quick Start: Python

The Python package (`context-sanitizer-py`) is pre-compiled using `maturin` with zero C/C++ grammar dependencies. You do not need Rust installed on your machine to use it.

### Installation

```bash
pip install context-sanitizer
```

### Usage

```python
from context_sanitizer import PromptSanitizer

# Initialize the engine with security, Shadow Vault, and configurable models
sanitizer = PromptSanitizer(
    use_system_one_guard=True,         # Enable local System One semantic firewall & routing
    use_ml_compression=True,           # Enable neural compression (Stage 3B)
    guard_model_id="MoritzLaurer/DeBERTa-v3-xsmall-mnli-fever-anli-ling-wanli", # Configurable!
    compression_model_id="microsoft/llmlingua-2-bert-base-multilingual-cased-meetingbank",
    target_compression_rate=0.33,      # Target rate: keep ~33% of the original tokens
    force_tokens=['\n', '?']           # Ensure structural and interrogative tokens are never dropped
)

dirty_prompt = """
System: You are an AI assistant.
User: Please check why alice@acme.com is getting an error during deployment.
Code: 
AWS_ACCESS_KEY_ID=<YOUR_AWS_ACCESS_KEY_ID>
function execute_deployment() {
    //  This is a   very    long     comment that explains nothing
    return true;
}
"""

# Scrubs root keys ([REDACTED_*]), vaults PII (<SHADOW:EMAIL_*>), compresses bloat, and runs System One guardrails
result = sanitizer.sanitize_and_compress(dirty_prompt)

# Handle security block or proceed with clean, compressed prompt
if result.get("status") == "blocked":
    print(f"⚠️ Security Alert: Payload blocked! Reason: {result['reason']}")
    print(f"Violations detected: {result.get('violations', [])}")
else:
    print("--- Compressed Prompt (Safe for Cloud LLM) ---")
    print(result["compressed_prompt"])

    # When the LLM emits a tool call using the shadow token, rehydrate locally using the session handle:
    llm_tool_call = '{"tool": "lookup_user", "email": "<SHADOW:EMAIL_8f3a2c>"}'
    local_tool_args = sanitizer.rehydrate(llm_tool_call, session_id=result["session_id"])
    print("\n--- Rehydrated Local Tool Call ---")
    print(local_tool_args)

    print("\n--- Optimization Metrics ---")
    print(f"Original Tokens: {result['origin_tokens']}")
    print(f"Compressed Tokens: {result['compressed_tokens']}")
    print(f"Compression Ratio: {result['ratio']} (Targeted 0.33)")
    print(f"Estimated API Savings: {result['saving']}")
```

## Quick Start: Rust

You can embed the pure Rust core directly into your own proxy servers or Rust-based AI agents, and swap models via builder configuration or the `SemanticGuard` and `TokenCompressor` traits.

### Installation

```bash
cargo add context-sanitizer
```

### Usage

```rust
use context_sanitizer::{SanitizerEngine, Decision};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configurable builder (swap Hugging Face or local model_id anytime)
    let engine = SanitizerEngine::builder()
        .with_target_compression_rate(0.35)
        .with_guard_model("MoritzLaurer/DeBERTa-v3-xsmall-mnli-fever-anli-ling-wanli")
        .with_compressor_model("microsoft/llmlingua-2-bert-base-multilingual-cased-meetingbank")
        .build()?;
    
    let dirty_payload = r#"
        {
            "role": "user",
            "content": "My GitHub token is <YOUR_GITHUB_TOKEN>. Check account alice@acme.com please."
        }
    "#;
    
    // Inspects payload: root keys -> [REDACTED_*], PII -> session Shadow Vault, syntax-locked compression
    match engine.inspect(dirty_payload) {
        Decision::Allowed(safe_payload) => {
            println!("Safe Payload for LLM:\n{}", safe_payload.content);

            // Rehydrate session-scoped PII tokens locally when executing tool calls
            let tool_args = r#"{"email": "<SHADOW:EMAIL_8f3a2c>"}"#;
            let rehydrated = safe_payload.vault_handle().rehydrate(tool_args);
            println!("Rehydrated Tool Args:\n{}", rehydrated);
        }
        Decision::Blocked(violation) => {
            eprintln!("⚠️ Security Alert: Payload blocked! [{}] {}", violation.category, violation.reason);
        }
    }

    Ok(())
}
```

## Agentic AI & Copilot UI Integrations

Modern Agentic workflows and **Copilot UIs (including CopilotKit and OpenAI-compatible APIs)** suffer from **context rot** (token bloat from recursive loops and verbose tool schemas), **user PII/secret exposure** (users pasting personal information or credentials into chat), and **adversarial code injection** (attackers injecting malicious payloads through frontend copilot widgets). 

`context-sanitizer` provides a **hardened two-tier defense model with a split secret policy**:
1. **Soft Mitigation (Session-Scoped Shadow Vault & Root-Secret Masking):**
   - **Operational PII** (emails, phones, IPs, user IDs) is swapped in-place with session-scoped cryptographic tokens (`<SHADOW:EMAIL_8f3a2c>`). The cloud LLM never sees raw PII, while local tool calls rehydrate the original values via `.rehydrate()` using the session's `VaultHandle`.
   - **Root Credentials** (AWS secret keys, GitHub PATs, private keys) default to one-way `[REDACTED_*]` masks so an indirect prompt injection can never trick an agent tool into exfiltrating root keys.
2. **Hard Mitigation (Circuit Breaker & Quarantine):** When the System One gatekeeper (`SemanticGuard`) or pure-Rust lexical scanner detects malicious code injections, proprietary semantic leaks, or jailbreaks, it short-circuits execution immediately—protecting your infrastructure and preventing costly third-party API calls.

| Scenario | System Action | Downstream LLM Invoked? |
| :--- | :--- | :--- |
| **Operational PII in Copilot UI** (`email`, `IP`, `phone`) | Swapped via **Session-Scoped Shadow Vault** (`<SHADOW:EMAIL_8f3a2c>`) | **Yes** (Safe payload forwarded; tool calls rehydrated via `.rehydrate()`) |
| **Root Credentials** (`AWS_KEY`, `GITHUB_TOKEN`, `PEM`) | Masked one-way with `[REDACTED_*]` tokens | **Yes** (Safe payload forwarded; root keys never rehydrated to tools) |
| **Token Bloat / Stale History / Tool Schemas** | Compressed via Pure-Rust Tier 2 & *LLMLingua-2* Tier 3 | **Yes** (Latency & cost reduced by 40–70%) |
| **Proprietary Semantic Leaks** | Blocked via Zero-Shot NLI + L0 custom terms | **No** (Short-circuited locally before API call) |
| **Malicious Code / Prompt Injections** | Quarantined to SecOps/SIEM | **No** (Zero inference cost incurred) |

---

### Python: LangGraph Integration (with Security Quarantine & Tool Rehydration)

Use LangGraph conditional edges (`add_conditional_edges`) to route execution to a `quarantine` node when severe security threats or semantic leaks are intercepted, and rehydrate Shadow-Vaulted PII locally before tool execution:

```python
from typing import TypedDict, List, Optional
from langchain_core.messages import BaseMessage, AIMessage
from langgraph.graph import StateGraph, END
from context_sanitizer import PromptSanitizer

# 1. Define Agent State with Security Flags & Vault Session ID
class AgentState(TypedDict):
    messages: List[BaseMessage]
    clean_prompt: Optional[str]
    vault_session_id: Optional[str]
    is_blocked: bool
    rejection_reason: Optional[str]

# 2. Initialize with Configurable System One Guardrails
sanitizer = PromptSanitizer(
    use_system_one_guard=True,
    guard_model_id="MoritzLaurer/DeBERTa-v3-xsmall-mnli-fever-anli-ling-wanli",
    blocked_categories=["PROPRIETARY_IP", "UNRELEASED_FINANCIALS", "PROMPT_INJECTION"],
    hard_block_on_threat=True,
    target_compression_rate=0.40
)

# 3. Node: Sanitize, Vault PII, Compress, and Evaluate Threat Level
def sanitize_node(state: AgentState):
    raw_context = "\n".join(f"{m.type}: {m.content}" for m in state["messages"])
    result = sanitizer.sanitize_and_compress(raw_context)
    
    # Check if System One Firewall tripped a hard block (short-circuited)
    if result.get("status") == "blocked":
        return {
            "is_blocked": True,
            "rejection_reason": result["reason"]
        }
        
    return {
        "clean_prompt": result["compressed_prompt"],
        "vault_session_id": result["session_id"],
        "is_blocked": False
    }

# 4. Node: Security Quarantine (Circuit Breaker - No LLM Call)
def quarantine_node(state: AgentState):
    # Log to SIEM / SecOps and safely alert user without API invocation
    alert_msg = f"Security Policy Violation: {state['rejection_reason']}"
    return {"messages": state["messages"] + [AIMessage(content=alert_msg)]}

# 5. Node: Standard Model Execution + Local Tool Call Rehydration
def call_model_node(state: AgentState):
    response = llm.invoke(state["clean_prompt"])
    # If the LLM emitted tool calls containing <SHADOW:*> tokens, rehydrate locally:
    if getattr(response, "tool_calls", None) and state.get("vault_session_id"):
        for call in response.tool_calls:
            call["args"] = sanitizer.rehydrate(call["args"], session_id=state["vault_session_id"])
    return {"messages": state["messages"] + [response]}

# 6. Build Conditional State Graph
def route_after_sanitization(state: AgentState) -> str:
    return "quarantine" if state["is_blocked"] else "agent"

workflow = StateGraph(AgentState)
workflow.add_node("sanitize", sanitize_node)
workflow.add_node("quarantine", quarantine_node)
workflow.add_node("agent", call_model_node)

workflow.set_entry_point("sanitize")
workflow.add_conditional_edges(
    "sanitize",
    route_after_sanitization,
    {
        "quarantine": "quarantine",
        "agent": "agent"
    }
)
workflow.add_edge("quarantine", END)
workflow.add_edge("agent", END)

app = workflow.compile()
```

---

### Rust: Rig Integration (Circuit Breaker & Shadow Vault Pattern)

In Rust-based agents built with [Rig](https://github.com/0xPlaygrounds/rig), use `Decision` matching to prevent tainted or malicious tool outputs from reaching the LLM and `.rehydrate()` to restore operational PII for local tools:

```rust
use context_sanitizer::{SanitizerEngine, Decision, SecurityViolation};
use rig::providers::openai::{self, GPT_4O};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = openai::Client::from_env();
    let engine = SanitizerEngine::builder()
        .with_guard_model("MoritzLaurer/DeBERTa-v3-xsmall-mnli-fever-anli-ling-wanli")
        .build()?;

    // Raw payload from runtime tool execution containing confidential data
    let dirty_tool_output = r#"
        System: Executing cloud audit for customer alice@acme.com...
        Tool [bash]: AWS_SECRET_ACCESS_KEY=<YOUR_AWS_SECRET_ACCESS_KEY>
        Tool [files]: Confidential Q4 Unreleased Financial Forecasts for Project Alpha.
    "#;

    // Evaluate input through the System One firewall, Shadow Vault, & secret engine
    match engine.inspect(dirty_tool_output) {
        Decision::Allowed(clean_payload) => {
            // Forward sanitized & compressed context to Rig agent
            let agent = client
                .agent(GPT_4O)
                .preamble("You are a secure cloud infrastructure assistant.")
                .build();

            let response = agent.prompt(&clean_payload.content).await?;
            // Rehydrate any <SHADOW:*> operational PII tokens if needed for local follow-up tools:
            let local_output = clean_payload.vault_handle().rehydrate(&response);
            println!("Rig Agent Response:\n{}", local_output);
        }
        Decision::Blocked(violation) => {
            // Hard circuit breaker triggered: DO NOT call the LLM
            eprintln!(
                "CRITICAL SECURITY ALERT: Blocked violation [{}] -> {}",
                violation.category, violation.reason
            );

            // Log incident to SIEM / abort execution safely
            return Err(format!("Halted: {}", violation.reason).into());
        }
    }

    Ok(())
}
```

## Build from Source

The repository is structured as a **3-crate Cargo workspace** (`100% pure Rust` by default with zero C/C++ grammar build dependencies):
* `crates/context-sanitizer`: Core Rust library (`L0` SIMD/lexical guard + Shadow Vault, `L0.5` statistical pruning, and `L1` `SemanticGuard` / `TokenCompressor` traits).
* `crates/context-sanitizer-cli`: The `context-sanitizer xray` terminal visualizer & benchmark binary.
* `crates/context-sanitizer-py`: `PyO3` / `maturin` Python bindings.

### Prerequisites

* **Rust**: Stable 1.75+ via [rustup](https://rustup.rs/)
* **Python**: 3.9+ (development headers and virtual environment)
* **cargo-maturin** *(Optional)*: `cargo install maturin` for packaging and editable installs

### 1. Clone the Repository

```bash
git clone https://github.com/reidlai/context-sanitizer.git
cd context-sanitizer
```

### 2. Build via Cargo

Compile the entire workspace (Rust library, proxy binary, and PyO3 extension) in release mode:

```bash
cargo build --release --workspace
```

Run the test suite across all crates:

```bash
cargo test --workspace
```

*(Optional)* Compile with local neural/ML compression features enabled:

```bash
cargo build --release --workspace --features ml
```

### 3. Build & Package Python Extension

You can package and install the Python bindings via the `cargo maturin` subcommand or compile directly using native `cargo build`:

#### Method A: Using `cargo maturin` (Recommended)

```bash
# Install cargo-maturin subcommand (one-time setup)
cargo install maturin

# Compile and link directly into your active Python environment
cargo maturin develop --release

# Or build standalone distributable wheels (.whl)
cargo maturin build --release
```

#### Method B: Pure `cargo build` (No Python build tooling)

Because PyO3 compiles to a standard C-ABI dynamic library (`cdylib`), you can build the extension using standard Cargo:

```bash
cargo build --release --features python
```

Copy or symlink the compiled shared object to your working directory or `PYTHONPATH`:
* **Linux:** `cp target/release/libcontext_sanitizer.so ./context_sanitizer.so`
* **macOS:** `cp target/release/libcontext_sanitizer.dylib ./context_sanitizer.so`
* **Windows:** `copy target\release\context_sanitizer.dll .\context_sanitizer.pyd`

## Contributing

Contributions are welcome! Direct pushes to the `main` branch are restricted to maintain codebase stability and strict security invariants. All contributors must follow the standard Fork & Pull Request workflow.

### Pull Request Workflow

1. **Fork the Repository:** Create your personal fork at [github.com/reidlai/context-sanitizer](https://github.com/reidlai/context-sanitizer).
2. **Create a Feature Branch:**
   ```bash
   git clone https://github.com/<your-username>/context-sanitizer.git
   cd context-sanitizer
   git checkout -b feature/your-feature-name
   ```
3. **Validate Changes:** Ensure all test suites, formatting, and lints pass:
   ```bash
   cargo test --workspace
   cargo fmt --check
   cargo clippy --workspace --all-targets -- -D warnings
   ```
4. **Submit a Pull Request (PR):**
   * Push your feature branch to your personal fork.
   * Open a PR against `reidlai/context-sanitizer:main`.
   * Detail your changes, rationale, and verification steps in the PR description.
5. **Review & Merge:**
   * Every PR will be inspected by an AI review agent and the repository maintainer for correctness, security bounds, and test fidelity.
   * Feedback will be shared in the PR review comments, and the branch will be merged once approved.

### Reporting Issues

Encountered a bug, security vulnerability, or have an idea for a new feature? Please [open an issue](https://github.com/reidlai/context-sanitizer/issues/new/choose) using our standardized GitHub Issue Templates:

* 🐛 **[Bug Report](https://github.com/reidlai/context-sanitizer/issues/new?template=bug_report.yml):** For reporting unexpected panics, compression defects, or runtime crashes.
* 💡 **[Feature Request](https://github.com/reidlai/context-sanitizer/issues/new?template=feature_request.yml):** For proposing new regex detectors, System One encoder models, or agent framework integrations.
* 🛡️ **[Security Vulnerability](https://github.com/reidlai/context-sanitizer/issues/new?template=security_report.yml):** For reporting secret detector bypasses, false negatives in proprietary data detection, or guardrail escapes.

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.
