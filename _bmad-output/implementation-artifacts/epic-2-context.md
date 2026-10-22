# Epic 2 Context: Zero-Weight Syntax-Safe Statistical Compression & Context X-Ray CLI (Tier 2 + CLI E2E)

<!-- Generated from planning artifacts. Regenerate with compile-epic-context if planning docs change. -->

## Goal

Developers and engineering leaders can compress multi-turn conversational contexts by 40%–70% and OpenAI JSON tool schemas by $\ge 30\%$ in `~300,000–400,000 ns` median (`< 1,500,000 ns` P99 even on adversarial compression-bomb payloads; `< 500,000 ns` combined L0+L0.5 median) with 0 MB of model weights, guaranteed 0% JSON/code syntax corruption via lexical token locking and the Fail-Open Syntax Invariant, and audit real-time nanosecond (`ns`) latency, token savings, and benchmark corpora via `context-sanitizer xray <payload> [--bench]`.

## Stories

- Story 2.1: Pure-Rust Lexical Syntax Token Locking, Anti-Smuggling Precedence & Fail-Open AST Invariant
- Story 2.2: Tail-Query Auto-Anchoring, Statistical Context Pruning & Anti-Compression-Bomb Knapsack Selection
- Story 2.3: OpenAI JSON Tool Schema Minification
- Story 2.4: Context X-Ray Diagnostic & Benchmarking CLI (`context-sanitizer xray`) & Tier 2 Post-Build E2E Suite

## Requirements & Constraints

- **Lexical Syntax Token Locking:** Ingest raw text and structured payloads to lexically tag all structural JSON tokens (`{`, `}`, `[`, `]`, `:`, `,`), object keys, markdown code fences (```` ``` ````), inline shell commands (`` `command` ``), subshells (`$(...)`), and environment variable expansions (`$VAR`, `${VAR}`) as immutable (`SpanKind::Locked`) in $O(N)$ linear time with zero C/C++ compiler dependencies.
- **Anti-Smuggling Precedence Invariant:** Redaction spans (`SpanKind::Redacted`, `SpanKind::Shadow`) hold absolute priority over `SpanKind::Locked`. Secrets and operational PII embedded within markdown code fences, shell commands, or JSON keys must remain 100% intercepted and redacted or vaulted to prevent syntax-lock secret smuggling attacks. Downstream statistical pruning must never mark any locked byte range as `SpanKind::Pruned`.
- **Fail-Open Syntax Invariant:** Following materialization of compressed payloads, structural and JSON validity must be verified (via `serde_json::from_str` and balanced delimiter checks). If validation fails, the materializer must fail open: discard all `SpanKind::Pruned` deletions for that block and emit the original uncompressed text while keeping all `SpanKind::Redacted` and `SpanKind::Shadow` substitutions permanently applied, guaranteeing 0% JSON parse errors in downstream LLM calls.
- **Tail-Query Auto-Anchoring:** The latest user query/message, system prompt directives (`role: "system"`), and active JSON schema keys must be assigned infinite priority (`SpanKind::Retained`) and never pruned, preserving critical conversational context and intent.
- **Statistical Context Pruning:** Score unlocked conversational history using static unigram Shannon self-information ($I(w) = -\log P(w)$) and query-anchored Okapi BM25 relevance against the latest user query. Apply $O(N \log N)$ lazy-greedy max-heap monotone submodular $(1 - 1/e)$ knapsack selection to mark low-significance filler tokens as `SpanKind::Pruned` until reaching the target compression ratio (default `rate = 0.60`, target range 40%–70%), maintaining reasoning accuracy within a $\le 1.5\%$ variance guardrail across benchmark tasks (`MeetingBank`, `GSM8K`).
- **Adversarial & Latency Budgets (Anti-Compression-Bomb):** Complete Tier 2 statistical pruning in `~300,000–400,000 ns` median (`~75–100 ns/byte`, `< 500,000 ns` combined Tier 1 + Tier 2 median) and strictly `< 1,500,000 ns` P99 per 4KB payload even on adversarial compression-bomb inputs (e.g., 2,000+ single-character tokens or deeply nested delimiters). Allocate zero persistent heap structures outside active sessions and require 0 MB of downloaded model weights.
- **OpenAI Tool Schema Minification:** Strip insignificant whitespace, optional parameter prose descriptions, and redundant metadata annotations from `"tools": [...]` JSON arrays while strictly preserving tool `name`, `parameters.type`, `properties` keys, parameter types, `enum` definitions, and `required` arrays, achieving $\ge 30\%$ tool schema token reduction while remaining 100% schema-valid for OpenAI, Anthropic, and local LLMs.
- **Context X-Ray CLI & Benchmarking Harness:** Provide a standalone CLI binary crate (`crates/context-sanitizer-cli`) with compiled release binary size $< 15$ MB. In diagnostic mode (`context-sanitizer xray <payload> [--rate <f32>]`), render an ASCII dashboard showing nanosecond latencies, token counts, pruning percentage, secret/PII counts, syntax lock preservation status, and estimated dollar savings (GPT-4o, Claude 3.5 Sonnet). In benchmark mode (`--bench`), run deterministic test suites (`MeetingBank`, `GSM8K`, synthetic credentials/PII, adversarial ReDoS/compression-bomb corpora), report throughput and P50/P99 latency, and exit with non-zero status if nanosecond SLA budgets regress.
- **Continuous DevSecOps CI & Coverage Enforcement:** Register `crates/context-sanitizer-cli` in the Cargo workspace. Stage 4 (Pre-Build) must execute all workspace unit tests and in-process Tier 1 + Tier 2 integration tests (`crates/context-sanitizer/tests/e2e_tier2_compression.rs`, including syntax-lock secret smuggling and fail-open recovery tests) with `cargo llvm-cov --workspace --all-targets --fail-under-lines 85` passing before release compilation. Stage 6 (Post-Build) must execute black-box CLI tests against the compiled release binary (`target/release/context-sanitizer xray`) across Linux, macOS, and Windows.

## Technical Decisions

- **Cargo Workspace Registration:** Register `crates/context-sanitizer-cli` under `[workspace.members]` in root `Cargo.toml`. The CLI crate depends unidirectionally on `crates/context-sanitizer` (Core); Core remains pure Rust with zero dependencies on the CLI or C/C++ toolchains.
- **Pinned Stack Dependencies:** Use `logos 0.16.1` for DFA lexical token scanning in Core, `clap 4.5.20` (with derive feature) in the CLI crate, along with `serde 1.0.210`, `serde_json 1.0.128`, and `thiserror 1.0.64`.
- **Core Compression Subsystem (`crates/context-sanitizer/src/compress/`):**
  - `mod.rs`: Public interface and orchestrator for Tier 2 compression passes.
  - `lexer.rs`: `logos`-based token lexer and bounded delimiter scanner that generates `SpanKind::Locked` for JSON syntax, markdown fences, shell backticks, subshells, and environment variables.
  - `shannon.rs`: Precomputed static unigram frequency tables and self-information calculations ($I(w) = -\log P(w)$) with zero runtime model overhead.
  - `bm25.rs`: Okapi BM25 relevance scorer ranking historical token spans against the tail user query.
  - `submodular.rs`: $O(N \log N)$ lazy-greedy max-heap monotone submodular $(1 - 1/e)$ knapsack optimizer selecting spans to mark as `SpanKind::Pruned`.
  - `schema.rs`: JSON Schema minifier that strips verbose descriptions while maintaining strict schema compliance.
- **CLI Subsystem (`crates/context-sanitizer-cli/src/`):**
  - `main.rs`: Entry point and `clap` CLI argument definitions.
  - `xray.rs`: Terminal rendering logic for the ASCII diagnostic dashboard, token analytics, and cost modeling (GPT-4o, Claude 3.5 Sonnet).
  - `bench.rs`: Automated micro-benchmarking runner against bundled test corpora.
- **Span Precedence & Materialization Invariant (`arena.rs`):**
  - `SpanKind::Redacted` and `SpanKind::Shadow` hold absolute precedence over `SpanKind::Locked`, preventing secret smuggling inside code blocks.
  - Unlocked spans marked as `SpanKind::Pruned` are omitted during final buffer construction.
  - If post-materialization validation fails on JSON structures, all `SpanKind::Pruned` deletions are discarded and raw text with redaction/shadow spans intact is emitted.

## UX & Interaction Patterns

- **CLI Invocation:**
  - `context-sanitizer xray <payload.json>`: Evaluates a target JSON payload at default compression rate (`0.60`).
  - `context-sanitizer xray <payload.json> --rate <f32>`: Adjusts target pruning aggressiveness (e.g., `--rate 0.50`).
  - `context-sanitizer xray <payload.json> --bench`: Executes automated benchmark suite against standard and adversarial corpora.
- **Terminal ASCII Dashboard:** Emits a formatted UTF-8/ASCII summary displaying:
  - Latency breakdown: Tier 1 runtime (`ns` and `ns/byte`), Tier 2 runtime (`ns` and `ns/byte`), total elapsed nanoseconds.
  - Token metrics: Original tokens, compressed tokens, percentage reduction.
  - Security summary: Count of redacted root credentials, vaulted PII shadow tokens, and syntax lock preservation rate (100%).
  - Economics: Estimated dollar savings per 1M tokens based on standard commercial LLM pricing tiers.
- **Exit Codes & Error Handling:** Return exit code `0` on clean analysis and successful benchmark validation. Return non-zero exit codes on file read errors, invalid CLI arguments, or nanosecond SLA benchmark regressions.

## Cross-Story Dependencies

- **Story 2.1 $\to$ Stories 2.2 & 2.4:** Lexical syntax locking (`compress/lexer.rs`) and fail-open validation establish the protected byte boundaries required before statistical pruning (`Story 2.2`) can safely mark removable spans, and provide lock integrity metrics for the CLI dashboard (`Story 2.4`).
- **Story 2.2 $\to$ Story 2.4:** Statistical pruning (`compress/shannon.rs`, `bm25.rs`, `submodular.rs`) provides the core token reduction engine measured and audited by `context-sanitizer xray`.
- **Story 2.3 $\to$ Stories 2.2 & 2.4:** Schema minification (`compress/schema.rs`) integrates with chat payload processing and contributes to overall token reduction reported by the CLI.
- **Epic 1 $\to$ Epic 2:** Epic 2 builds directly upon Epic 1's `Span` arena (`arena.rs`), `VaultHandle` (`guard/vault.rs`), OpenAI wire serialization (`wire/`), and 6-stage DevSecOps CI pipeline.
- **Epic 2 $\to$ Epics 3–5:** The Tier 2 statistical compressor provides the automatic fallback target for Epic 3's ML gatekeeper (`SemanticGuard`), and forms the core compression engine exported to Python (`crates/context-sanitizer-py` in Epic 4) and WebAssembly (`wasm/` in Epic 5).
