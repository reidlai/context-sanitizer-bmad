# context-sanitizer

**context-sanitizer** is a blazingly fast, local Rust proxy and library that intercepts LLM API traffic to act as a **System One semantic firewall** and token compressor.

Built for modern Agentic AI and Copilot workflows, it solves the "context rot" problem by trimming stale history and massive tool schemas, while ensuring your AWS keys, GitHub tokens, and proprietary semantic data never leak to third-party LLM providers. It offers a pure Rust core with seamless Python bindings via `PyO3`.

## The Hybrid Architecture

This project splits the prompt engineering pipeline into two distinct layers for maximum performance and security:

1. **The System One Firewall:** Uses blazing-fast, open-source System One models (e.g., DeBERTa-v3, ShieldGemma) to make bounded decisions (blocking semantic leaks and routing payloads) in a single forward pass without generating text.
2. **The Rust & ML Compression Engine:** Leverages pure Rust lexical pruning and optional neural token classification to physically shrink the payload token-by-token.

## Research Foundation

This project implements state-of-the-art token pruning and semantic architectures based on recent developments in prompt engineering:

* **System One Inference for Bounded Semantic Reasoning**: Shifts away from sequential token generation, allowing non-autoregressive, encoder-only architectures (like Microsoft's DeBERTa-v3) to compute text semantics in a single pass for zero-latency routing and safety checks.
* **LLMLingua-2: Data Distillation for Efficient and Faithful Task-Agnostic Prompt Compression (2024)**: Microsoft's research formulates prompt compression as a token classification problem. Using a small BERT-sized encoder, it achieves 3x-6x faster compression and accelerates end-to-end latency by 1.6x-2.9x while minimizing performance loss.
* **Prompt Compression for Large Language Models: A Survey (2024)**: Establishes the academic taxonomy of prompt compression, dividing the field into "hard" compression (discrete token removal/pruning) and "soft" compression (trainable continuous vectors).

## Development Methodology

This project is engineered using **Spec-Driven Development (SDD)** guided by the **BMAD Method**.

Rather than ad-hoc implementation, all architecture and capabilities follow a disciplined, specification-first engineering lifecycle:

* **Specification-First Architecture**: Interface contracts, C-ABI boundaries (`PyO3`), PII/secret regex invariants, and System One guardrails are strictly formalized before code implementation.
* **Bounded Multi-Agent Execution**: Complex pipelines (such as hybrid lexical pruning vs. neural classification) are decomposed into discrete, bounded tasks with explicit validation criteria.
* **Deterministic Verification**: Specifications directly drive automated test suites across both the Rust workspace and Python bindings to guarantee zero secret leakage, predictable compression ratios, and cross-language memory safety.

## Features

* **System One Semantic Firewall**: Utilizes fast, open-weights encoder models to evaluate payloads natively, blocking proprietary business concepts or semantic leaks before they leave your machine.
* **Hard Token Compression (Human-Readable)**: Implements "hard" compression techniques to physically remove redundant tokens. Unlike soft prompt vectors, this ensures the compressed payload remains fully human-readable for security auditing and black-box API compatibility.
* **Zero-Leak Regex Security**: Sub-millisecond interception of hardcoded secrets using the `aho-corasick` multi-pattern search engine.
* **Intelligent Payload Routing**: Local classifications instantly route different context types (e.g., "Code", "Logs", "Conversational") to the most efficient pruning algorithms.
* **Lexical Pruning**: Automatically strips markdown bloat, minimizes JSON tool schemas, and truncates stale conversational history natively in Rust.
* **Opt-In ML Compression**: Built-in hooks to support Hugging Face's `candle` framework for local, token-by-token neural compression.
* **Cross-Language Support**: Publish as a native Rust crate or install directly into your Python Agentic loops.

## Quick Start: Python

The Python package is pre-compiled using `maturin`. You do not need Rust installed on your machine to use it.

### Installation

```bash
pip install context-sanitizer
```

### Usage

```python
from context_sanitizer import PromptSanitizer

# Initialize the engine with security and optimization parameters
sanitizer = PromptSanitizer(
    use_system_one_guard=True,         # Enable local open-source semantic firewall & routing
    use_ml_compression=True,           # Enable neural compression (e.g., LLMLingua-2)
    target_compression_rate=0.33,      # Target rate: keep ~33% of the original tokens
    force_tokens=['\n', '?']           # Ensure structural and interrogative tokens are never dropped
)

dirty_prompt = """
System: You are an AI assistant.
User: Please check my code for the upcoming deployment. I am getting an error.
Code: 
AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE
function execute_deployment() {
    //  This is a   very    long     comment that explains nothing
    //  Project Alpha Launch Strategy: Unreleased Q4 Financials
    return true;
}
"""

# The engine scrubs credentials, compresses bloat, and checks semantic guardrails
result = sanitizer.sanitize_and_compress(dirty_prompt)

# Handle security block or proceed with clean, compressed prompt
if result.get("status") == "blocked":
    print(f"⚠️ Security Alert: Payload blocked! Reason: {result['reason']}")
    print(f"Violations detected: {result.get('violations', [])}")
else:
    print("--- Compressed Prompt ---")
    print(result["compressed_prompt"])

    print("\n--- Optimization Metrics ---")
    print(f"Original Tokens: {result['origin_tokens']}")
    print(f"Compressed Tokens: {result['compressed_tokens']}")
    print(f"Compression Ratio: {result['ratio']} (Targeted 0.33)")
    print(f"Estimated API Savings: {result['saving']}")
```

## Quick Start: Rust

You can embed the pure Rust core directly into your own proxy servers or Rust-based AI agents.

### Installation

```bash
cargo add context-sanitizer
```

### Usage

```rust
use context_sanitizer::{SanitizerEngine, Decision};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = SanitizerEngine::new();
    
    let dirty_payload = r#"
        {
            "role": "user",
            "content": "My GitHub token is ghp_1234567890abcdef.    Please help   me."
        }
    "#;
    
    // Inspects payload for secrets, applies semantic guardrails, and compresses tokens
    match engine.inspect(dirty_payload) {
        Decision::Allowed(safe_payload) => {
            println!("Safe Payload:\n{}", safe_payload.content);
        }
        Decision::Blocked(violation) => {
            eprintln!("⚠️ Security Alert: Payload blocked! [{}] {}", violation.category, violation.reason);
        }
    }

    Ok(())
}
```

## Agentic AI Integrations

Modern Agentic workflows suffer from **context rot** (token bloat from recursive loops and verbose tool schemas) and **secret exposure** (leaking credentials or proprietary data from tool execution outputs). 

`context-sanitizer` provides a **two-tier defense model**:
1. **Soft Mitigation (Redaction & Masking):** Standard secrets (e.g., AWS keys, GitHub PATs) are sanitized in-place with `[REDACTED_*]` tokens so the agent can safely continue.
2. **Hard Mitigation (Circuit Breaker & Quarantine):** When the System One firewall detects proprietary semantic leaks, jailbreaks, or fraud, it halts execution immediately—protecting sensitive IP and preventing costly third-party API calls.

| Scenario | System Action | Downstream LLM Invoked? |
| :--- | :--- | :--- |
| **Standard Keys / Credentials** | Masked with `[REDACTED_*]` tokens | **Yes** (Safe payload forwarded) |
| **Token Bloat / Stale History** | Compressed lexically & neurally | **Yes** (Latency & cost reduced) |
| **Proprietary Semantic Leaks** | Blocked with security reason | **No** (Halted locally before API call) |
| **Prompt Injections / Fraud** | Quarantined to SecOps/SIEM | **No** (Zero inference cost incurred) |

---

### Python: LangGraph Integration (with Security Quarantine)

Use LangGraph conditional edges (`add_conditional_edges`) to route execution to a `quarantine` node when severe security threats or semantic leaks are intercepted:

```python
from typing import TypedDict, List, Optional
from langchain_core.messages import BaseMessage, AIMessage
from langgraph.graph import StateGraph, END
from context_sanitizer import PromptSanitizer

# 1. Define Agent State with Security Flags
class AgentState(TypedDict):
    messages: List[BaseMessage]
    clean_prompt: Optional[str]
    is_blocked: bool
    rejection_reason: Optional[str]

# 2. Initialize with System One Guardrails
sanitizer = PromptSanitizer(
    use_system_one_guard=True,
    blocked_categories=["PROPRIETARY_IP", "UNRELEASED_FINANCIALS", "PROMPT_INJECTION"],
    hard_block_on_threat=True,
    target_compression_rate=0.40
)

# 3. Node: Sanitize, Compress, and Evaluate Threat Level
def sanitize_node(state: AgentState):
    raw_context = "\n".join(f"{m.type}: {m.content}" for m in state["messages"])
    result = sanitizer.sanitize_and_compress(raw_context)
    
    # Check if System One Firewall tripped a hard block
    if result.get("status") == "blocked":
        return {
            "is_blocked": True,
            "rejection_reason": result["reason"]
        }
        
    return {
        "clean_prompt": result["compressed_prompt"],
        "is_blocked": False
    }

# 4. Node: Security Quarantine (Circuit Breaker - No LLM Call)
def quarantine_node(state: AgentState):
    # Log to SIEM / SecOps and safely alert user without API invocation
    alert_msg = f"Security Policy Violation: {state['rejection_reason']}"
    return {"messages": state["messages"] + [AIMessage(content=alert_msg)]}

# 5. Node: Standard Model Execution
def call_model_node(state: AgentState):
    response = llm.invoke(state["clean_prompt"])
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

### Rust: Rig Integration (Circuit Breaker Pattern)

In Rust-based agents built with [Rig](https://github.com/0xPlaygrounds/rig), use `Decision` matching to prevent tainted or malicious tool outputs from reaching the LLM:

```rust
use context_sanitizer::{SanitizerEngine, Decision, SecurityViolation};
use rig::providers::openai::{self, GPT_4O};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = openai::Client::from_env();
    let engine = SanitizerEngine::new();

    // Raw payload from runtime tool execution containing confidential data
    let dirty_tool_output = r#"
        System: Executing cloud audit...
        Tool [bash]: AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
        Tool [files]: Confidential Q4 Unreleased Financial Forecasts for Project Alpha.
    "#;

    // Evaluate input through the System One firewall & secret engine
    match engine.inspect(dirty_tool_output) {
        Decision::Allowed(clean_payload) => {
            // Forward sanitized & compressed context to Rig agent
            let agent = client
                .agent(GPT_4O)
                .preamble("You are a secure cloud infrastructure assistant.")
                .build();

            let response = agent.prompt(&clean_payload.content).await?;
            println!("Rig Agent Response:\n{}", response);
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

All components (pure Rust core, proxy binary, and Python bindings) are driven entirely through standard `cargo` workflows.

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

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.
