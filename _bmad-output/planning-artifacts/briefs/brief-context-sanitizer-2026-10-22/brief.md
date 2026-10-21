---
title: "Product Brief: context-sanitizer"
status: review
created: 2026-10-22
updated: 2026-10-22
---

# Product Brief: context-sanitizer

## Executive Summary

**context-sanitizer** is a high-performance, local-first Rust library, CLI, and Python binding that intercepts LLM and agentic API traffic to serve as a **System One semantic firewall** and **syntax-safe token compressor**. Released as an open-source project under the permissive MIT license, it tackles the twin bottlenecks crippling modern AI workflows: **context rot** (runaway costs, latency degradation, and schema bloat) and **boundary security vulnerabilities** (secret exfiltration, PII exposure, and indirect prompt injection).

Rather than relying on sluggish, recursive LLM guard calls or heavy Python runtimes that add 100ms+ of overhead, `context-sanitizer` introduces a **3-Tier Speculative Cascade**. It delivers sub-millisecond deterministic redaction (`< 0.5 ms`), zero-weight statistical pruning (`< 2 ms`, requiring 0 MB of downloaded model weights), and optional non-autoregressive encoder classification (via pluggable traits). 

For developers, `context-sanitizer` provides a zero-friction, drop-in utility that accelerates agents and drastically slashes token expenditures without breaking code syntax or JSON schemas. For senior engineering leadership and technical evaluators, it serves as a showcase of systems programming in Rust, sound AI safety threat modeling, and the rigor of Spec-Driven Development (SDD) via the BMAD method.

---

## The Problem: The Cost, Latency, and Security Crisis at the Context Boundary

Modern Agentic AI, Copilot frontends, and Retrieval-Augmented Generation (RAG) pipelines place unsustainable pressure on context windows:

1. **Context Rot and Dollar Bloat:** Massive JSON tool definitions, repetitive message histories, and low-entropy conversational filler congest LLM contexts. This degrades reasoning accuracy ("lost in the middle"), inflates API bills by 40%–70%, and drags down user perceived responsiveness.
2. **Security & Privacy Exposure:** Autonomous agents handle live user inputs and system environments. A single indirect prompt injection can coax an LLM into dumping embedded AWS secrets, GitHub tokens, or proprietary customer data into tool arguments or remote logging sinks.
3. **The Guardrail Dilemma (Slow vs. Fragile):** Existing guardrail solutions fall into two flawed extremes:
   - *Heavyweight System 2 / Python Rails:* Tools like NeMo Guardrails, Presidio, or hosted cloud APIs introduce 50ms–500ms of latency, require multi-gigabyte Python environments with complex C dependencies, and introduce network round-trips.
   - *Naive Regex & Ad-Hoc Truncation:* Brittle string replacements break valid code ASTs, invalidate strict JSON payloads, and blindly redact operational PII (such as user email addresses) that downstream agent tools legitimately require to complete their tasks.

---

## The Solution: 3-Tier Speculative Cascade

`context-sanitizer` sits directly at the ingestion boundary (supporting standard OpenAI-compatible `/v1/chat/completions` formats and Copilot payloads) and executes a staged pipeline designed for maximum speed and safety:

```text
Incoming Agent / Copilot Payload
   │
   ▼
┌───────────────────────────────────────────────────────────────────────────┐
│ Tier 1 (L0): Pure-Rust Deterministic & Lexical Guard (< 0.5 ms | Always)  │
│  • Aho-Corasick SIMD pattern matching for high-throughput secret detection │
│  • Split-Secret Policy: One-way masks for root keys; Shadow Vault for PII │
│  • logos + serde_json pure-Rust lexical scanner locks code/JSON syntax   │
└───────────────────────────────────────────────────────────────────────────┘
   │
   ▼
┌───────────────────────────────────────────────────────────────────────────┐
│ Tier 2 (L0.5): Pure-Rust Smart Statistical Pruning (< 2 ms | 0 MB Weights)│
│  • Unigram Shannon self-information / IDF + Okapi BM25 relevance scoring  │
│  • Tail-query auto-anchoring & submodular chunk selection                 │
│  • Schema minification & conversational history deduplication             │
└───────────────────────────────────────────────────────────────────────────┘
   │
   ▼
┌───────────────────────────────────────────────────────────────────────────┐
│ Tier 3 (L1): Configurable Trait-Based System One ML (--features ml)       │
│  • Pluggable SemanticGuard: Fast encoder classification (DeBERTa / ModernBERT)│
│    Short-circuits malicious prompt/code injections before compression runs│
│  • Pluggable TokenCompressor: LLMLingua-2 BERT token classification       │
└───────────────────────────────────────────────────────────────────────────┘
```

### Core Product Capabilities

1. **Session-Scoped "Shadow Vault" + Split Secret Policy:**
   - *Root Credentials (`AWS_KEY`, `GITHUB_TOKEN`, `PRIVATE_KEY`):* Default to irreversible `[REDACTED_*]` masks so indirect prompt injections can never exfiltrate active credentials.
   - *Operational PII (`EMAIL`, `IP`, `PHONE`):* Swapped with session-scoped cryptographic shadow tokens (e.g., `<SHADOW:EMAIL_8f3a2c>`) tied to a local `VaultHandle`. Developers use `.rehydrate()` locally so legitimate tool calls (e.g., querying a user's account) remain functional.
2. **Zero-Weight Pure-Rust Compression (Tier 2):**
   - Implements information-theoretic pruning using static unigram Shannon self-information ($I(w) = -\log P(w)$) and Okapi BM25 with tail-query auto-anchoring.
   - Operates in under 2ms with **zero model weights downloaded** (0 MB), stripping conversational filler and minifying schemas safely.
3. **Syntax-Safe Token Locking:**
   - A pure-Rust lexical scanner (`logos` + `serde_json`) locks JSON syntax and code delimiters before compression algorithms touch the text, guaranteeing 0% JSON parse failures in downstream LLM calls.
4. **Context X-Ray Diagnostic CLI (`context-sanitizer xray`):**
   - A terminal benchmarking and inspection utility that visualizes vaulted PII, redacted keys, pruned tokens, System 1 threat scores, dollar savings, and microsecond latency.

---

## What Makes This Different

| Feature / Trait | Legacy Python Guardrails / hosted APIs | context-sanitizer | Why It Matters |
| :--- | :--- | :--- | :--- |
| **Execution Latency** | 50 ms – 500 ms (often cloud network round-trip) | **< 0.5 ms (L0) / < 2 ms (L0.5)** | Sub-millisecond execution keeps interactive copilots fluid and realtime. |
| **Model Weights & Overhead** | Heavy PyTorch / ONNX runtimes (1GB – 4GB+) | **0 MB weights by default** (Tier 1 & 2) | Runs in resource-constrained environments, CI/CD, or edge nodes instantly. |
| **Agent Tool Usability** | Blind redaction breaks tool function calls | **Two-Way Shadow Vault (`.rehydrate()`)** | Preserves operational PII for downstream tools without exposing data to the LLM. |
| **JSON & Code Safety** | Pruning algorithms corrupt JSON formatting | **Pure-Rust Syntax Token Locking** | Code blocks and tool schemas remain 100% syntactically valid. |
| **Deployment Footprint** | C/C++ compilation chains, fragile virtualenvs | **Single Rust binary / pre-compiled wheels** | Zero-friction installation across Rust (`cargo`) and Python (`pip`). |
| **Development Rigor** | Ad-hoc heuristics | **BMAD Spec-Driven Architecture** | Formally defined interface contracts and verified safety invariants. |

---

## Target Audience & Personas

1. **The AI Systems / Agent Developer (The Practitioner):**
   - *Context:* Building production copilots, RAG agents, or multi-turn assistants using frameworks like LangGraph, CopilotKit, or direct OpenAI endpoints.
   - *Needs:* A drop-in filter that cuts token bills and prevents PII leaks without adding latency, breaking JSON tool calls, or requiring heavy GPU setup.
   - *Success:* Adds `context-sanitizer` via Cargo or pip in 5 minutes; observes immediate 50%+ token reductions and verified secret redaction with zero syntax errors.
2. **Senior Technical Leadership & Architecture Evaluators (The Reviewer):**
   - *Context:* CTOs, VPs of Engineering, and Staff+ Architects evaluating technical competency, security posture, and systems design quality.
   - *Needs:* Evidence of deep domain knowledge across systems programming, LLM context engineering, and software delivery discipline.
   - *Success:* Recognizes state-of-the-art encoder threat modeling, pure-Rust zero-dependency design decisions, and clean Spec-Driven Development (SDD) traceability.

---

## Scope & Boundaries

### In-Scope (Version 1.0 Release)
- **3-Crate Cargo Workspace:**
  - `context-sanitizer`: Core pure-Rust library exposing public API and pluggable traits (`SemanticGuard`, `TokenCompressor`).
  - `context-sanitizer-cli`: Standalone binary shipping the terminal-based `xray` benchmark/diagnostic tool.
  - `context-sanitizer-py`: PyO3-based Python bindings distributed as pre-compiled wheels with zero C/C++ toolchain dependencies.
- **Tier 1 (L0):** SIMD `aho-corasick` secret/PII scanning, session-scoped cryptographic shadow vault, `.rehydrate()` API, and `logos` syntax locking.
- **Tier 2 (L0.5):** Zero-weight Shannon self-information/IDF, Okapi BM25 pruning, tail-query auto-anchoring, and JSON schema minification.
- **Tier 3 (L1 - `--features ml`):** Off-the-shelf encoder inference using `candle-transformers` (supporting `DeBERTa-v3` / `ModernBERT` and `LLMLingua-2` BERT).
- **OpenAI & Copilot Payload Compatibility:** Ingests standard `/v1/chat/completions` message arrays and tool schema objects. `[ASSUMPTION]`

### Out-of-Scope (Explicit Non-Goals)
- **Hosted Cloud / SaaS Backend:** Strictly local-first library and CLI; no proprietary cloud telemetry, mandatory remote auth, or hosted servers.
- **Training Foundation Models:** No bespoke neural weights or model distillation; operates purely with 0-weight mathematical algorithms or off-the-shelf Hugging Face checkpoints.
- **Autoregressive Generative Rails:** Will not invoke slow System 2 LLMs (e.g., asking GPT-4 "Evaluate if this message is safe") at the boundary.
- **Soft Prompt / Continuous Embedding Pruning:** Exclusively focused on hard prompt compression (discrete tokens) to ensure broad model compatibility.

---

## Success Criteria & Key Performance Indicators (KPIs)

- **Performance & Latency:**
  - Tier 1 (L0 Guard): `< 0.5 ms` per 4KB payload.
  - Tier 2 (L0.5 Statistical Compressor): `< 2.0 ms` per 4KB payload.
  - Tier 3 (L1 Neural, CPU/Metal): `< 25 ms` forward pass for System 1 classification.
- **Compression Efficiency:**
  - 40%–70% token reduction on bloated multi-turn histories and large JSON schemas.
  - 0% JSON parsing failures or code syntax invalidation downstream.
- **Security & Integrity:**
  - 100% interception of known high-entropy root credential patterns (AWS, GitHub, private keys).
  - Cryptographically isolated session shadow vaults with zero cross-tenant token collision.
- **Developer Adoption & Portability:**
  - `pip install context-sanitizer-py` and `cargo add context-sanitizer` install with 0 external C/C++ build requirements.
  - Context X-Ray CLI provides instant, visually compelling verification out of the box.

---

## Long-Term Vision (2–3 Years)

If successful, `context-sanitizer` establishes itself as the default, lightweight **standard context boundary layer** for autonomous agents and client-side copilots:
- **Universal Wasm/WASI Compilation:** Compiling Tier 1 and Tier 2 directly into WebAssembly, enabling zero-latency client-side sanitization inside web browsers, Electron apps, and edge CDN workers before data touches the internet.
- **Native Ecosystem Middleware:** Built-in first-class integrations across modern agent frameworks (LangGraph, CrewAI, AutoGen, CopilotKit) and local inference runtimes (Ollama, vLLM).
- **Industry Benchmark for SDD:** Recognized across open-source communities as an exemplary reference project proving how Spec-Driven Development and BMAD yield bulletproof, verified systems software.
