---
title: "PRFAQ Distillate: context-sanitizer"
type: llm-distillate
source: "prfaq-context-sanitizer.md"
created: "2026-10-22"
purpose: "Token-efficient context for downstream PRD creation"
---

# PRFAQ Distillate: context-sanitizer

## 1. Product Positioning & Value Proposition
- **Project Identity:** `context-sanitizer` is an open-source (MIT license), local-first Rust library, Python binding, and CLI tool designed as a System One semantic firewall and syntax-safe token compressor.
- **Dual Stakeholder Goal:**
  - *Practitioners (AI/LLM Engineers):* Solves context rot and dollar inflation (40%–70% savings) while stopping credential and PII exfiltration without corrupting JSON tool calls or adding UI lag.
  - *Evaluators (Senior Tech Leadership):* Serves as an engineering showcase of systems programming in Rust, sound AI safety threat modeling, and Spec-Driven Development (BMAD).
- **Core Architecture:** 3-Tier Speculative Cascade:
  - *Tier 1 (L0):* Pure-Rust deterministic guard (`< 0.5 ms`), Aho-Corasick SIMD pattern matching, Split-Secret Policy (one-way root credential masking vs. two-way session shadow vault with local `.rehydrate()`), and `logos` + `serde_json` syntax token locking.
  - *Tier 2 (L0.5):* Pure-Rust statistical pruning (`< 2 ms`, 0 MB model download) via unigram Shannon self-information/IDF, Okapi BM25, and tail-query auto-anchoring.
  - *Tier 3 (L1):* Pluggable trait-based System One ML (`--features ml`) running non-autoregressive encoder classifiers (`DeBERTa-v3`, `ModernBERT`, `LLMLingua-2` BERT) via `candle-transformers`.

## 2. Scope & Boundaries (In vs. Out)
- **Strictly In-Scope for V1:**
  - 3-crate Cargo workspace: `context-sanitizer` (core lib), `context-sanitizer-cli` (`xray` CLI binary), and `context-sanitizer-py` (`PyO3` / `maturin` pre-compiled wheels).
  - OpenAI `/v1/chat/completions` message arrays and tool JSON schema compatibility.
  - Split-Secret Policy: Permanent one-way masks (`[REDACTED_*]`) for root keys; session-scoped cryptographic hex shadow tokens (`<SHADOW:TYPE_hash>`) for PII with thread-isolated `VaultHandle`.
  - Terminal X-Ray diagnostic tool visualizer.
- **Explicitly Out-of-Scope (Non-Goals):**
  - Hosted cloud SaaS backend or remote telemetry; no proprietary logins or network tracking.
  - Training custom foundation models or weight distillation.
  - Autoregressive System 2 generative guardrails (e.g., calling GPT-4 to judge prompt safety).
  - Soft continuous embedding prompt compression; strictly hard discrete token compression.

## 3. Rejected Alternatives & Technical Rationale
- **Rejected C-Based `tree-sitter` Grammars:** Discarded due to C compilation fragility on Windows/macOS and friction with precompiled `maturin` wheels. Replaced with pure-Rust `logos` lexer and `serde_json` token locking.
- **Rejected Single Shared-Backbone Model:** Hugging Face offers no unified checkpoint sharing `[CLS]` sequence classification (SentencePiece 128k) and token pruning heads (WordPiece 119k). Distilling bespoke models is unnecessary given sub-millisecond Tier 2 performance.
- **Rejected Rehydrating Root Credentials:** Rehydrating root cloud keys (`AWS_KEY`) enables indirect prompt injection exfiltration via tool arguments. Root keys must always remain one-way masked.
- **Rejected Hosted SaaS Proxy Architecture:** Cloud proxies add 50ms–200ms internet latency and violate data sovereignty mandates requiring secrets to be redacted before leaving the local boundary.

## 4. Competitive Intelligence & Differentiators
- **vs. NeMo Guardrails / Presidio / LLM Guard:** Legacy Python solutions add 100ms–500ms latency, require gigabytes of dependencies, and often fail on JSON tool schemas. `context-sanitizer` runs in `< 2 ms` in pure Rust with 0 MB model weights.
- **vs. Microsoft LLMLingua-2:** LLMLingua-2 requires PyTorch and GPU for low latency. `context-sanitizer` provides weightless Tier 2 statistical compression out of the box with optional Candle-based Tier 3 fallback.
- **vs. Hosted Gateways (Lakera / Cloudflare AI Gateway):** `context-sanitizer` runs 100% locally on the host machine, guaranteeing zero network latency and complete privacy compliance.

## 5. Downstream Action Items & Risk Mitigations (From The Verdict)
- **Syntax Fail-Open Invariant (Critical):** If complex nested JSON or pathological multibyte UTF-8 input fails validation after pruning, the engine must fail-open by emitting the original uncompressed block rather than risking an unparseable payload.
- **Turnkey Agent Middleware (Needs More Heat):** Build lightweight Python middleware adapters for LangGraph, CrewAI, AutoGen, and CopilotKit to provide zero-boilerplate adoption.
- **Automated Public Benchmark Harness (Needs More Heat):** Commit reproducible CI benchmark scripts running against MeetingBank and GSM8K to validate accuracy and latency claims publicly.
