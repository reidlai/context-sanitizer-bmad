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

# The proxy will block/redact the semantic leak (Project Alpha), 
# scrub the AWS key, and compress token bloat.
result = sanitizer.sanitize_and_compress(dirty_prompt)

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
use context_sanitizer::SanitizerEngine;

fn main() {
    let engine = SanitizerEngine::new();
    
    let dirty_payload = r#"
        {
            "role": "user",
            "content": "My GitHub token is ghp_1234567890abcdef.    Please help   me."
        }
    "#;
    
    // Scans for secrets and compresses repetitive tokens
    let safe_payload = engine.process(dirty_payload);
    
    println!("Safe Payload: {}", safe_payload);
}
```

## Agentic AI Integrations

Modern Agentic workflows suffer from **context rot** (token bloat from recursive loops and large tool schemas) and **secret exposure** (leaking API keys from command/tool outputs). `context-sanitizer` fits directly into agent loops in both Python and Rust.

### Python: LangGraph Integration

Intercept and compress message history and tool outputs inside a LangGraph `StateGraph` node before calling the LLM:

```python
from typing import TypedDict, List
from langchain_core.messages import BaseMessage
from langgraph.graph import StateGraph, END
from context_sanitizer import PromptSanitizer

# 1. Define Agent State
class AgentState(TypedDict):
    messages: List[BaseMessage]
    clean_prompt: str

# 2. Initialize context-sanitizer
sanitizer = PromptSanitizer(
    use_system_one_guard=True,     # Block semantic leaks & malicious prompt injections
    use_ml_compression=True,       # Enable token classification pruning
    target_compression_rate=0.40   # Compress down to 40% of original tokens
)

# 3. Node: Sanitize and Compress Context
def sanitize_node(state: AgentState):
    # Formulate accumulated conversation and tool execution history
    raw_context = "\n".join(f"{m.type}: {m.content}" for m in state["messages"])
    
    # Intercept credentials, strip schema bloat, and compress
    result = sanitizer.sanitize_and_compress(raw_context)
    
    return {"clean_prompt": result["compressed_prompt"]}

# 4. Node: Call Model
def call_model_node(state: AgentState):
    # Model receives concise, credential-free prompt
    response = llm.invoke(state["clean_prompt"])
    return {"messages": state["messages"] + [response]}

# 5. Build Graph
workflow = StateGraph(AgentState)
workflow.add_node("sanitize", sanitize_node)
workflow.add_node("agent", call_model_node)
workflow.set_entry_point("sanitize")
workflow.add_edge("sanitize", "agent")
workflow.add_edge("agent", END)

app = workflow.compile()
```

### Rust: Rig Integration

In Rust-based agents built with [Rig](https://github.com/0xPlaygrounds/rig), use `SanitizerEngine` to scrub runtime tool outputs and stale context before prompting the agent:

```rust
use context_sanitizer::SanitizerEngine;
use rig::providers::openai::{self, GPT_4O};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = openai::Client::from_env();
    let sanitizer = SanitizerEngine::new();

    // Raw, bloated payload from multi-step tool calls and terminal executions
    let dirty_tool_output = r#"
        System: Executing cloud resource audit...
        Tool [bash]: AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
        Tool [db_query]: {"columns": ["id", "pii_ssn", "name"], "rows": [["1", "000-12-3456", "Alice"]]}
        Status: Finished query with redundant whitespace and repetitive log lines.
    "#;

    // 1. Scrub credentials and prune token bloat in sub-millisecond Rust execution
    let safe_context = sanitizer.process(dirty_tool_output);

    // 2. Build Rig Agent
    let agent = client
        .agent(GPT_4O)
        .preamble("You are an autonomous cloud infrastructure engineer.")
        .build();

    // 3. Prompt agent with protected, compressed payload
    let response = agent.prompt(&safe_context).await?;
    println!("Rig Agent Response:\n{}", response);

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
