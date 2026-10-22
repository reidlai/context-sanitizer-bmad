---
title: "Product Brief: context-sanitizer"
status: final
created: 2026-10-22
updated: 2026-10-22
---

# Product Brief: context-sanitizer

## Executive Summary

**context-sanitizer** is a high-performance, local-first Rust library, CLI, Python binding (`context-sanitizer-py`), and WebAssembly package (`@context-sanitizer/wasm`) that intercepts LLM and agentic API traffic to serve as a **System One semantic firewall** and **syntax-safe token compressor**. Released as an open-source project under the permissive MIT license, it tackles the twin bottlenecks crippling modern AI workflows: **context rot** (runaway costs, latency degradation, and schema bloat) and **boundary security vulnerabilities** (secret exfiltration, PII exposure, and indirect prompt injection).

Rather than relying on sluggish, recursive LLM guard calls or heavy Python runtimes that add 100,000,000+ ns (100ms+) of overhead, `context-sanitizer` introduces a **3-Tier Speculative Cascade**. It delivers nanosecond-scale deterministic redaction (`~20,000–100,000 ns` median, `< 500,000 ns` P99), zero-weight statistical pruning (`~300,000–400,000 ns` median, `< 1,500,000 ns` P99, requiring 0 MB of downloaded model weights), and optional non-autoregressive encoder classification (`< 100,000 ns` lexical fast-breaker + `< 5,000,000 ns` P99 ceiling via pluggable traits).

For developers, `context-sanitizer` provides a zero-friction, drop-in utility that accelerates agents and drastically slashes token expenditures without breaking code syntax or JSON schemas. For senior engineering leadership and technical evaluators, it serves as a showcase of systems programming in Rust, sound AI safety threat modeling, and the rigor of Spec-Driven Development (SDD) via the BMAD method.

---

## The Problem: The Cost, Latency, and Security Crisis at the Context Boundary

Modern Agentic AI, Copilot frontends, and Retrieval-Augmented Generation (RAG) pipelines place unsustainable pressure on context windows:

1. **Context Rot and Dollar Bloat:** Massive JSON tool definitions, repetitive message histories, and low-entropy conversational filler congest LLM contexts. This degrades reasoning accuracy ("lost in the middle"), inflates API bills by 40%–70%, and drags down user perceived responsiveness.
2. **Security & Privacy Exposure:** Autonomous agents handle live user inputs and system environments. A single indirect prompt injection can coax an LLM into dumping embedded AWS secrets, GitHub tokens, or proprietary customer data into tool arguments or remote logging sinks.
3. **The Guardrail Dilemma (Slow vs. Fragile):** Existing guardrail solutions fall into two flawed extremes:
   - *Heavyweight System 2 / Python Rails:* Tools like NeMo Guardrails, Presidio, or hosted cloud APIs introduce 50,000,000–500,000,000 ns (50ms–500ms) of latency, require multi-gigabyte Python environments with complex C dependencies, and introduce network round-trips.
   - *Naive Regex & Ad-Hoc Truncation:* Brittle string replacements break valid code ASTs, invalidate strict JSON payloads, and blindly redact operational PII (such as user email addresses) that downstream agent tools legitimately require to complete their tasks.

---

## The Solution: 3-Tier Speculative Cascade

`context-sanitizer` sits directly at the ingestion boundary (supporting standard OpenAI-compatible `/v1/chat/completions`, `/v1/chat`, and `/v1/completions` formats and Copilot payloads) and executes a staged pipeline designed for maximum speed and safety:

```text
Incoming Agent / Copilot Payload
   │
   ▼
┌───────────────────────────────────────────────────────────────────────────┐
│ Tier 1 (L0): Pure-Rust Deterministic & Lexical Guard (< 500,000 ns P99)   │
│  • Aho-Corasick O(N) SIMD pattern matching (~20,000–100,000 ns median)    │
│  • Split-Secret Policy: [REDACTED_CREDENTIAL] for keys; Shadow Vault PII  │
│  • logos + serde_json pure-Rust lexical scanner locks code/JSON syntax   │
└───────────────────────────────────────────────────────────────────────────┘
   │
   ▼
┌───────────────────────────────────────────────────────────────────────────┐
│ Tier 2 (L0.5): Pure-Rust Smart Statistical Pruning (< 1,500,000 ns | 0 MB)│
│  • Unigram Shannon self-information / IDF + Okapi BM25 (~300k–400k ns)   │
│  • Tail-query auto-anchoring & O(N log N) submodular chunk selection      │
│  • Schema minification & conversational history deduplication             │
└───────────────────────────────────────────────────────────────────────────┘
   │
   ▼
┌───────────────────────────────────────────────────────────────────────────┐
│ Tier 3 (L1): Configurable Trait-Based System One ML (--features ml)       │
│  • Pluggable SemanticGuard: < 100,000 ns fast-breaker + < 5,000,000 ns    │
│    top-K surprisal window encoder (Prompt-Guard-2 / DeBERTa-v3)           │
│  • Pluggable TokenCompressor: LLMLingua-2 BERT token classification       │
└───────────────────────────────────────────────────────────────────────────┘
```

### Core Product Capabilities

1. **Session-Scoped "Shadow Vault" + Split Secret Policy:**
   - *Root Credentials (`AWS_KEY`, `GITHUB_TOKEN`, `PRIVATE_KEY`):* Default to irreversible `[REDACTED_CREDENTIAL]` masks so indirect prompt injections can never exfiltrate active credentials.
   - *Operational PII (`EMAIL`, `IP`, `PHONE`):* Swapped with session-scoped cryptographic shadow tokens (e.g., `<SHADOW:EMAIL_8f3a2c>`) tied to a local `VaultHandle`. Developers use `vault.rehydrate(args, destination="...")` locally with destination whitelisting so legitimate tool calls remain functional.
2. **Zero-Weight Pure-Rust Compression (Tier 2):**
   - Implements information-theoretic pruning using static unigram Shannon self-information ($I(w) = -\log P(w)$) and Okapi BM25 with tail-query auto-anchoring.
   - Operates in `~300,000–400,000 ns` median (`< 1,500,000 ns` P99) with **zero model weights downloaded** (0 MB), stripping conversational filler and minifying schemas safely.
3. **Syntax-Safe Token Locking:**
   - A pure-Rust lexical scanner (`logos` + `serde_json`) locks JSON syntax and code delimiters before compression algorithms touch the text, guaranteeing 0% JSON parse failures in downstream LLM calls via a strict Fail-Open Syntax Invariant.
4. **Context X-Ray Diagnostic CLI (`context-sanitizer xray`):**
   - A terminal benchmarking and inspection utility that visualizes vaulted PII, redacted keys, pruned tokens, System 1 threat scores, dollar savings, and nanosecond (`ns`) latency (`ns/byte` histograms).

---

## What Makes This Different

| Feature / Trait | Legacy Python Guardrails / hosted APIs | context-sanitizer | Why It Matters |
| :--- | :--- | :--- | :--- |
| **Execution Latency** | 50,000,000 – 500,000,000 ns (50ms–500ms) | **`< 500,000 ns` (L0) / `< 1,500,000 ns` (L0.5) P99** | Nanosecond-scale execution keeps interactive copilots fluid and resistant to ReDoS/DoS. |
| **Model Weights & Overhead** | Heavy PyTorch / ONNX runtimes (1GB – 4GB+) | **0 MB weights by default** (Tier 1 & 2) | Runs in resource-constrained environments, CI/CD, or edge nodes instantly. |
| **Agent Tool Usability** | Blind redaction breaks tool function calls | **Two-Way Shadow Vault (`.rehydrate()`)** | Preserves operational PII for whitelisted tools without exposing data to the LLM. |
| **JSON & Code Safety** | Pruning algorithms corrupt JSON formatting | **Pure-Rust Syntax Token Locking** | Code blocks and tool schemas remain 100% syntactically valid. |
| **Deployment Footprint** | C/C++ compilation chains, fragile virtualenvs | **Single Rust binary / pre-compiled wheels / Wasm** | Zero-friction installation across Rust (`cargo`), Python (`pip`), and WebAssembly (`npm`). |
| **Development Rigor** | Ad-hoc heuristics | **BMAD Spec-Driven Architecture & 6-Stage DevSecOps** | Formally defined interface contracts, `.pre-commit-config.yaml` hooks, and $\ge 85\%$ coverage gates. |

---

## Target Audience & Personas

1. **The AI Systems / Agent Developer (The Practitioner):**
   - *Context:* Building production copilots, RAG agents, or multi-turn assistants using frameworks like LangGraph, CrewAI, AutoGen, CopilotKit, or direct OpenAI endpoints.
   - *Needs:* A drop-in filter that cuts token bills and prevents PII leaks without adding latency, breaking JSON tool calls, or requiring heavy GPU setup.
   - *Success:* Adds `context-sanitizer` via Cargo, pip, or npm in 5 minutes; observes immediate 50%+ token reductions and verified secret redaction with zero syntax errors.
2. **Senior Technical Leadership & Architecture Evaluators (The Reviewer):**
   - *Context:* CTOs, VPs of Engineering, and Staff+ Architects evaluating technical competency, security posture, and systems design quality.
   - *Needs:* Evidence of deep domain knowledge across systems programming, LLM context engineering, and software delivery discipline.
   - *Success:* Recognizes state-of-the-art encoder threat modeling, pure-Rust zero-dependency design decisions, and clean Spec-Driven Development (SDD) traceability.

---

## Scope & Boundaries

### In-Scope (Version 1.0 Release)
- **3-Crate Cargo Workspace + WebAssembly Package:**
  - `context-sanitizer`: Core pure-Rust library exposing public API and pluggable traits (`SemanticGuard`, `TokenCompressor`).
  - `context-sanitizer-cli`: Standalone binary shipping the terminal-based `xray` benchmark/diagnostic tool (`context-sanitizer xray --bench`).
  - `context-sanitizer-py`: PyO3-based Python bindings distributed as pre-compiled wheels with zero C/C++ toolchain dependencies and turnkey wrappers (`LangChainSanitizer`, `CrewAISanitizer`, `AutoGenSanitizer`).
  - `@context-sanitizer/wasm`: Universal WebAssembly / WASI compilation (`wasm-pack`) for browser and edge runtimes.
- **Tier 1 (L0):** $O(N)$ SIMD `aho-corasick` secret/PII scanning, `[REDACTED_CREDENTIAL]` masking, session-scoped cryptographic shadow vault, destination-whitelisted `.rehydrate()` API, and `logos` syntax locking.
- **Tier 2 (L0.5):** Zero-weight Shannon self-information/IDF, Okapi BM25 pruning, $O(N \log N)$ submodular selection, tail-query auto-anchoring, and JSON schema minification.
- **Tier 3 (L1 - `--features ml`):** `< 100,000 ns` Stage 3A-Fast lexical injection breaker + off-the-shelf encoder inference using `candle-transformers` over top-$K$ ($K \le 2$) surprisal windows (`< 5,000,000 ns` `guard_timeout_ns` P99 ceiling).
- **OpenAI & Copilot Payload Compatibility:** Ingests standard `/v1/chat/completions`, `/v1/chat`, and `/v1/completions` payloads.
- **Automated DevSecOps CI/CD & Pre-Commit Pipeline (`FR-14`):** Two-speed `.pre-commit-config.yaml` hooks and 6-stage GitHub Actions pipeline enforcing formatting, linting, SAST, SCA, $\ge 85\%$ test coverage prior to artifact builds, and post-build black-box E2E integration verification.

### Out-of-Scope (Explicit Non-Goals)
- **Hosted Cloud / SaaS Backend:** Strictly local-first library and CLI; no proprietary cloud telemetry, mandatory remote auth, or hosted servers.
- **Training Foundation Models:** No bespoke neural weights or model distillation; operates purely with 0-weight mathematical algorithms or off-the-shelf Hugging Face checkpoints.
- **Autoregressive Generative Rails:** Will not invoke slow System 2 LLMs (e.g., asking GPT-4 "Evaluate if this message is safe") at the boundary.
- **Soft Prompt / Continuous Embedding Pruning:** Exclusively focused on hard prompt compression (discrete tokens) to ensure broad model compatibility.

---

## Success Criteria & Key Performance Indicators (KPIs)

- **Performance & Nanosecond Latency SLAs (per 4KB payload):**
  - Tier 1 (L0 Guard): `~20,000–100,000 ns` median (`~5–25 ns/byte`), `< 500,000 ns` P99 ceiling.
  - Tier 2 (L0.5 Statistical Compressor): `~300,000–400,000 ns` median (`~75–100 ns/byte`), `< 1,500,000 ns` P99 ceiling (`< 500,000 ns` combined L0+L0.5 median).
  - Tier 3 (L1 Neural, CPU/Metal): `< 100,000 ns` (`~10–25 ns/byte`) Stage 3A-Fast lexical breaker; `~1,500,000–3,500,000 ns` median (`< 5,000,000 ns` hard `guard_timeout_ns` P99 ceiling) for System 1 neural classification.
- **Compression Efficiency:**
  - 40%–70% token reduction on bloated multi-turn histories and $\ge 30\%$ on large JSON tool schemas.
  - 0% JSON parsing failures or code syntax invalidation downstream.
- **Security & Integrity:**
  - 100% interception of known high-entropy root credential patterns (`[REDACTED_CREDENTIAL]`).
  - Cryptographically isolated session shadow vaults with zero cross-tenant token collision and strict destination whitelisting.
- **Developer Adoption & Portability:**
  - `pip install context-sanitizer-py`, `cargo add context-sanitizer`, and `@context-sanitizer/wasm` install with 0 external C/C++ build requirements.
  - Context X-Ray CLI provides instant, reproducible nanosecond verification out of the box.

---

## Long-Term Vision (2–3 Years)

If successful, `context-sanitizer` evolves from a v1 batch payload boundary library into the universal **streaming context security fabric** for autonomous systems:
- **Stateful Streaming Token Chunk Compression (v2+):** Extending lexical syntax locking and submodular pruning across live SSE/WebSocket streaming chunks.
- **Distributed Multi-Node Enterprise Vault Mesh (v2+):** Optional zero-knowledge encrypted vault synchronization across clustered inference gateways (Ollama, vLLM, Envoy).
- **Industry Benchmark for SDD:** Recognized across open-source communities as an exemplary reference project proving how Spec-Driven Development and BMAD yield bulletproof, verified systems software.
