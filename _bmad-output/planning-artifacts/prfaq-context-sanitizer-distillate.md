---
title: "PRFAQ Distillate: context-sanitizer"
type: llm-distillate
source: "prfaq-context-sanitizer.md"
created: "2026-10-22"
updated: "2026-10-22"
purpose: "Token-efficient context for downstream PRD creation"
---

# PRFAQ Distillate: context-sanitizer

## 1. Product Positioning & Value Proposition
- **Project Identity:** `context-sanitizer` is an open-source (MIT license), local-first Rust library, Python binding (`context-sanitizer-py`), WebAssembly package (`@context-sanitizer/wasm`), and CLI tool (`context-sanitizer-cli`) designed as a System One semantic firewall and syntax-safe token compressor.
- **Dual Stakeholder Goal:**
  - *Practitioners (AI/LLM Engineers):* Solves context rot and dollar inflation (40%–70% savings) while stopping credential and PII exfiltration without corrupting JSON tool calls or adding UI lag.
  - *Evaluators (Senior Tech Leadership):* Serves as an engineering showcase of systems programming in Rust, sound AI safety threat modeling, and Spec-Driven Development (BMAD).
- **Core Architecture:** 3-Tier Speculative Cascade:
  - *Tier 1 (L0):* Pure-Rust deterministic guard (`~20,000–100,000 ns` median / `< 500,000 ns` P99 per 4KB), $O(N)$ Aho-Corasick SIMD pattern matching, Split-Secret Policy (one-way `[REDACTED_CREDENTIAL]` root credential masking vs. two-way session shadow vault with destination-whitelisted `.rehydrate()`), and `logos` + `serde_json` syntax token locking.
  - *Tier 2 (L0.5):* Pure-Rust statistical pruning (`~300,000–400,000 ns` median / `< 1,500,000 ns` P99 per 4KB, `< 500,000 ns` combined L0+L0.5 median, 0 MB model download) via unigram Shannon self-information/IDF, Okapi BM25, $O(N \log N)$ submodular selection, and tail-query auto-anchoring.
  - *Tier 3 (L1):* Pluggable trait-based System One ML (`--features ml`) combining a `< 100,000 ns` (`~10–25 ns/byte`) $O(N)$ SIMD fast-breaker with non-autoregressive encoder classifiers (`Llama-Prompt-Guard-2-22M`, `DeBERTa-v3`, `ModernBERT`, `LLMLingua-2` BERT) via `candle-transformers` over top-$K$ ($K \le 2$) surprisal windows (`~1,500,000–3,500,000 ns` median, `< 5,000,000 ns` hard `guard_timeout_ns` P99 ceiling).

## 2. Scope & Boundaries (In vs. Out)
- **Strictly In-Scope for V1:**
  - 3-crate Cargo workspace + Wasm package: `context-sanitizer` (core lib), `context-sanitizer-cli` (`xray` CLI binary), `context-sanitizer-py` (`PyO3` / `maturin` pre-compiled wheels + turnkey `LangChainSanitizer`, `CrewAISanitizer`, `AutoGenSanitizer`), and `@context-sanitizer/wasm`.
  - OpenAI `/v1/chat/completions`, `/v1/chat`, and `/v1/completions` wire compatibility.
  - Split-Secret Policy: Permanent one-way masks (`[REDACTED_CREDENTIAL]`) for root keys; session-scoped cryptographic hex shadow tokens (`<SHADOW:TYPE_HEX6>`) for PII with thread-isolated `VaultHandle` and destination whitelisting.
  - Terminal X-Ray diagnostic visualizer (`context-sanitizer xray --bench`) with nanosecond (`ns`) latency reporting and 6-stage DevSecOps CI + `.pre-commit-config.yaml` pipeline (`FR-14`).
- **Explicitly Out-of-Scope (Non-Goals):**
  - Hosted cloud SaaS backend or remote telemetry; no proprietary logins or network tracking.
  - Training custom foundation models or weight distillation.
  - Autoregressive System 2 generative guardrails (e.g., calling GPT-4 to judge prompt safety).
  - Soft continuous embedding prompt compression; strictly hard discrete token compression.

## 3. Rejected Alternatives & Technical Rationale
- **Rejected C-Based `tree-sitter` Grammars:** Discarded due to C compilation fragility on Windows/macOS and friction with precompiled `maturin` wheels. Replaced with pure-Rust `logos` lexer and `serde_json` token locking.
- **Rejected Single Shared-Backbone Model:** Hugging Face offers no unified checkpoint sharing `[CLS]` sequence classification (SentencePiece 128k) and token pruning heads (WordPiece 119k). Distilling bespoke models is unnecessary given nanosecond-scale Tier 2 performance.
- **Rejected Rehydrating Root Credentials:** Rehydrating root cloud keys (`AWS_KEY`) enables indirect prompt injection exfiltration via tool arguments. Root keys must always remain one-way masked (`[REDACTED_CREDENTIAL]`).
- **Rejected Hosted SaaS Proxy Architecture:** Cloud proxies add 50,000,000–200,000,000 ns (50ms–200ms) internet latency and violate data sovereignty mandates requiring secrets to be redacted before leaving the local boundary.

## 4. Competitive Intelligence & Differentiators
- **vs. NeMo Guardrails / Presidio / LLM Guard:** Legacy Python solutions add 100,000,000–500,000,000 ns (100ms–500ms) latency, require gigabytes of dependencies, and often fail on JSON tool schemas. `context-sanitizer` runs in `< 500,000 ns` combined median (`< 1,500,000 ns` P99) in pure Rust with 0 MB model weights.
- **vs. Microsoft LLMLingua-2:** LLMLingua-2 requires PyTorch and GPU for low latency. `context-sanitizer` provides weightless Tier 2 statistical compression out of the box with optional Candle-based Tier 3 fallback.
- **vs. Hosted Gateways (Lakera / Cloudflare AI Gateway):** `context-sanitizer` runs 100% locally on the host machine, guaranteeing zero network latency and complete privacy compliance.

## 5. Downstream Action Items & Risk Mitigations (From The Verdict)
- **Syntax Fail-Open Invariant (Critical):** If complex nested JSON or pathological multibyte UTF-8 input fails validation after pruning, the engine must fail-open by emitting the original uncompressed block rather than risking an unparseable payload.
- **Turnkey Agent Middleware (Promoted to V1 MVP - `FR-11`):** Lightweight Python middleware adapters (`LangChainSanitizer`, `CrewAISanitizer`, `AutoGenSanitizer`) for zero-boilerplate adoption.
- **Automated Public Benchmark & DevSecOps Harness (Promoted to V1 MVP - `FR-12`, `FR-14`):** Reproducible CI benchmark scripts running against MeetingBank, GSM8K, and adversarial anti-DoS corpora alongside a 6-stage DevSecOps pipeline.
