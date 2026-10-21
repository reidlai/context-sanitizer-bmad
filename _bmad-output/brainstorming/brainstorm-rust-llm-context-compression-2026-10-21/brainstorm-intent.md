# Brainstorm Intent: `context-sanitizer`

## 1. Project Goal & Positioning
- **Objective:** Free, open-source Rust crate and Python library (`PyO3`) unifying **System One security gatekeeping**, **reversible PII/secret pseudonymization**, and **research-backed token compression** for LLM and agentic workflows.
- **Portfolio Value:** Showcases systems-level Rust (SIMD, zero-copy parsing, AST analysis, lock-free concurrency) combined with applied LLM context engineering and neural encoder inference.
- **Packaging:** Zero-weight pure-Rust default (`cargo add context-sanitizer` / `pip install context-sanitizer` with 0 MB model downloads) plus opt-in `--features ml` for lazy HuggingFace Hub (`hf-hub`) auto-pull of ~45 MB Int8 quantized weights.

## 2. Target Boundary (`Copilot UI` / `CopilotKit` / OpenAI API)
- **Boundary Defense:** Intercepts user inputs, RAG chunks, and MCP/tool JSON responses entering through Copilot UIs (e.g., CopilotKit) and OpenAI-compatible `/v1/chat/completions` payloads (`messages[]`, `tools[]`, `tool_calls[]`) before they reach external LLMs or agent memory.
- **Deployment Modes:**
  - In-process Rust crate / Python SDK.
  - Drop-in OpenAI/CopilotKit reverse-proxy sidecar (`axum` + `tower`).
- **Native Wire-Protocol Engine:** Zero-copy streaming JSON sanitizer (`simd-json` / `serde`) that prunes bloated tool schemas, multi-turn history, and irrelevant MCP array elements in-flight.

## 3. The 3-Tier Speculative Cascade
Resolves the latency-vs-intelligence contradiction by filtering deterministically first and invoking neural inference only on ambiguous spans:

1. **`L0` Pure-Rust SIMD / AST Guard (Sub-millisecond, Default):**
   - `aho-corasick` + SIMD regex for deterministic secret/PII scrubbing.
   - `tree-sitter` AST parsing to detect malicious code-injection patterns and lock structural syntax tokens in code/JSON blocks so syntax is never broken during pruning.
2. **`L0.5` Pure-Rust Information-Theoretic Pruning (0 MB Weights, Default):**
   - `tiktoken-rs` + BM25, TF-IDF, self-information entropy, and submodular chunk selection.
   - Delivers 40–60% hard token pruning on natural language, comments, docstrings, and tool outputs with zero external model dependencies.
3. **`L1` Shared-Backbone System One Neural Engine (`--features ml`, Opt-in):**
   - Single encoder forward pass (Int8 quantized ModernBERT / DeBERTa-v3-xs via `Candle` or `ORT`) driving three heads simultaneously:
     1. `[CLS]` Pooling Head: Hacking/prompt-injection gatekeeping and payload routing.
     2. Token-Level Pruning Head: LLMLingua-2 keep/drop token classification.
     3. Token-Level NER Head: Contextual PII/secret span detection.
   - **Surgical Threat Excision:** Excises only the injected/adversarial token spans from large RAG/tool payloads while compressing and forwarding the safe remainder.

## 4. Two-Way Ephemeral Shadow Vault
- **Mechanism:** Replaces detected PII and secrets with deterministic, format-preserving typed placeholders (`<PII:EMAIL_1>`, `<SECRET:AWS_KEY_1>`) stored in an in-memory Rust `DashMap` with strict TTL.
- **Agent Tool-Call Continuity:** External LLMs reason and emit `tool_calls` using the safe placeholders; `context-sanitizer` transparently rehydrates the real values locally right before local tool/MCP execution.

## 5. Flagship Demo: `context-sanitizer xray` CLI & Terminal Visualizer
- **Command:** `context-sanitizer xray`
- **Output:** Colorized terminal diff highlighting pruned tokens (red), Shadow-Vaulted PII/secrets (yellow), System One threat attribution scores, exact token compression ratios, and microsecond latency metrics.
- **Purpose:** Immediate visual proof of Rust performance and context-engineering efficacy for benchmarks and GitHub documentation.

## 6. Research Grounding
- **Token Classification Pruning:** LLMLingua-2 bidirectional encoder hard token pruning (Pan et al., ACL 2024).
- **Information-Theoretic Redundancy:** Selective Context / self-information token scoring, BM25, and submodular selection.
- **Structure-Preserving Code/JSON Compression:** `tree-sitter` AST structural locking combined with natural-language span pruning.
- **Multi-Task System One Gatekeeping:** Shared-backbone encoder attribution for indirect prompt-injection excision, NER pseudonymization, and context compression in a single pass.
