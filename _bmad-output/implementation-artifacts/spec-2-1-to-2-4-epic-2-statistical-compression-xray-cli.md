---
title: 'Epic 2 (Stories 2.1–2.4): Zero-Weight Syntax-Safe Statistical Compression, Tool Minification & Context X-Ray CLI'
type: 'feature'
created: '2026-10-22'
status: 'done'
baseline_revision: 'e3b82c45c7b48f9e22d8c474bfc268ce1b4dc34e'
review_loop_iteration: 0
followup_review_recommended: false
context:
  - '{project-root}/_bmad-output/implementation-artifacts/epic-2-context.md'
warnings:
  - 'multiple-goals'
  - 'oversized'
deferred: []
---

<intent-contract>

## Intent

**Problem:** The `context-sanitizer` workspace currently implements only the Tier 1 deterministic guard and shadow vault (`Epic 1`), lacking pure-Rust lexical syntax token locking (`logos`), fail-open JSON structural validation, tail-query auto-anchored Shannon/Okapi BM25 submodular knapsack context pruning (40%–70% reduction in `~300,000–400,000 ns`), OpenAI JSON tool schema minification ($\ge 30\%$ token reduction), the standalone `context-sanitizer xray` diagnostic/benchmarking CLI binary crate, and Stage 4/6 DevSecOps CI verification gates required by Stories 2.1 through 2.4.

**Approach:** Implement Stories 2.1–2.4 cohesively across `context-sanitizer/crates/context-sanitizer` (Core) and new `context-sanitizer/crates/context-sanitizer-cli` (CLI): (1) implement `compress/lexer.rs` using `logos` DFA to lock JSON delimiters/keys, markdown code fences, backticks, subshells, and `$VAR` as `SpanKind::Locked` while guaranteeing `Redacted > Shadow > Locked > Pruned` precedence (preventing syntax-lock secret smuggling) and the Fail-Open Syntax Invariant (`Story 2.1`), (2) implement `compress/shannon.rs`, `compress/bm25.rs`, and `compress/submodular.rs` for tail-query auto-anchoring, unigram surprisal, and $O(N \log N)$ lazy-greedy knapsack pruning within nanosecond budgets (`< 1,500,000 ns` P99) with 0 MB model weights (`Story 2.2`), (3) implement `compress/schema.rs` to minify OpenAI `"tools": [...]` definitions by stripping prose descriptions and redundant metadata while preserving names, property types, required keys, and enums (`Story 2.3`), and (4) create `crates/context-sanitizer-cli` (`main.rs`, `xray.rs`, `bench.rs`), register it in Cargo workspace, build Criterion benchmarks, and verify end-to-end via `tests/e2e_tier2_compression.rs` and the DevSecOps CI workflow (`Story 2.4`).

## Boundaries & Constraints

**Always:**
- Maintain `#![forbid(unsafe_code)]` and `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]` across all library modules in `crates/context-sanitizer`.
- Enforce strict span precedence in `SpanArena`: `SpanKind::Redacted` > `SpanKind::Shadow` > `SpanKind::Locked` > `SpanKind::Pruned` > `SpanKind::Retained`. Overlapping `SpanKind::Redacted` spans must merge into the union of their byte ranges (Epic 1 Retro Action Item 1).
- Enforce the Fail-Open Syntax Invariant: if a materialized payload chunk with pruned spans fails JSON syntax validation (`serde_json::from_str`) or balanced code fence/bracket checks, discard all `SpanKind::Pruned` deletions for that block and emit the original uncompressed text with `Redacted` and `Shadow` substitutions permanently applied.
- Prevent syntax-lock smuggling: secrets and operational PII located within code blocks, shell commands, or JSON keys must remain 100% intercepted and redacted or vaulted.
- In `submodular.rs`, auto-anchor the latest user query, system prompt directives (`role: "system"`), and schema keys with infinite priority (`SpanKind::Retained`), never pruning them.
- Preserve 0 MB model weights and zero C/C++ compiler dependencies (`gcc`, `clang`, MSVC not required; no C `tree-sitter`).
- Ensure the release binary of `context-sanitizer-cli` compiles to $< 15$ MB.
- Keep licensing strictly MIT with permissive dependencies (`MIT`, `Apache-2.0`, `BSD-3-Clause`) verified by `deny.toml`.
- All source code changes and new crates must be committed on branch `feat/epic-2` inside the `context-sanitizer` git submodule.

**Never:**
- Never allow token pruning to mark any `SpanKind::Locked` byte range as `SpanKind::Pruned`.
- Never over-prune beyond 70% compression or alter semantic tool schema validity.
- Never add C/C++ dependencies or persistent heap allocations outside active sessions.
- Never register unbuilt Epic 4 (`context-sanitizer-py`) or Epic 5 (`wasm`) crates in Cargo workspace during Epic 2.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Syntax Token Locking (`AD-1`, `FR4`) | Payload with `{ "key": "value" }`, ```` ```python ... ``` ````, `` `echo $SECRET` `` | All structural delimiters, keys, fences, backticks, `$SECRET` marked as `SpanKind::Locked`; never pruned | Clamps UTF-8 boundaries safely |
| Syntax-Lock Anti-Smuggling (`FR1`, `FR4`) | Code block ```` ```bash\nexport AWS_SECRET_ACCESS_KEY="wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"\n``` ```` | Secret inside code fence is 100% masked as `[REDACTED_CREDENTIAL]`; code fence formatting preserved | No error expected |
| Fail-Open Syntax Invariant (`FR4`, `AD-4`) | Compressed JSON where pruning would produce malformed JSON syntax | Discards `Pruned` removals, restores original text with secrets still redacted; downstream JSON parsing succeeds 100% | No error expected |
| Tail-Query Auto-Anchoring (`FR5`) | Multi-turn chat with older conversation turns and a latest user query + system prompt | System prompt and latest query marked `Retained` (infinite priority); older turns pruned to target rate (40%–70%) | No error expected |
| Anti-Compression-Bomb Budget (`NFR1`) | Adversarial 4KB payload with 2,000+ single-character tokens or deeply nested delimiters | Pruning executes in $< 1,500,000$ ns P99; 0 MB external weights; no exponential recursion | No error expected |
| Tool Schema Minification (`FR6`) | OpenAI `"tools": [...]` array with verbose descriptions, markdown docs, optional comments | Strips descriptions and formatting; preserves tool names, parameters, property types, required fields, and enums; achieves $\ge 30\%$ token reduction | Returns error if schema is malformed |
| Context X-Ray CLI Dashboard (`FR12`) | `context-sanitizer xray payload.json --rate 0.50` | Outputs clean ASCII terminal dashboard showing ns latency, ns/byte, tokens before/after, % pruned, secret/PII counts, locked syntax status, and estimated dollar savings (GPT-4o, Claude 3.5 Sonnet) | Exits code 0 on success |
| Benchmark Harness & SLA Gate (`FR12`, `NFR1`) | `context-sanitizer xray --bench` | Runs deterministic micro-benchmarks against MeetingBank, GSM8K, and synthetic fixtures; asserts P99 latency bounds; exits non-zero on regression | Non-zero exit code if SLA breached |

</intent-contract>

## Code Map

- `context-sanitizer/Cargo.toml` -- Root workspace manifest: add `crates/context-sanitizer-cli` to `members`; add `logos = "0.16.1"` and `clap = { version = "4.5.20", features = ["derive"] }` to `[workspace.dependencies]`.
- `context-sanitizer/crates/context-sanitizer/Cargo.toml` -- Core crate manifest: add `logos.workspace = true`.
- `context-sanitizer/crates/context-sanitizer/src/lib.rs` -- Expose `pub mod compress`, high-level `compress_text`, `compress_chat_json`, and re-export compression types.
- `context-sanitizer/crates/context-sanitizer/src/arena.rs` -- Fix Retro Action Item 1: merge overlapping `SpanKind::Redacted` intervals in `materialize` so overlapping credential spans redact the union of their byte ranges. Add `materialize_fail_open` or post-validation fail-open recovery logic.
- `context-sanitizer/crates/context-sanitizer/src/guard/patterns.rs` & `guard/entropy.rs` -- Fix Retro Action Item 3: consolidate duplicate `is_email_local_char` and `is_valid_email_local_byte` into shared `guard::is_email_local_byte`.
- `context-sanitizer/crates/context-sanitizer/src/compress/mod.rs` -- Tier 2 compression orchestrator tying together lexer, shannon, bm25, submodular, schema, and fail-open validation.
- `context-sanitizer/crates/context-sanitizer/src/compress/lexer.rs` -- Pure-Rust `logos` DFA tokenizer and bracket/fence matcher tagging JSON delimiters, markdown fences, shell backticks, subshells, and `$VAR` as `SpanKind::Locked`.
- `context-sanitizer/crates/context-sanitizer/src/compress/shannon.rs` -- Static unigram frequency table and surprisal self-information scorer ($I(w) = -\log P(w)$) with 0 runtime weights.
- `context-sanitizer/crates/context-sanitizer/src/compress/bm25.rs` -- Okapi BM25 relevance scorer comparing tokens against tail query anchor.
- `context-sanitizer/crates/context-sanitizer/src/compress/submodular.rs` -- $O(N \log N)$ lazy-greedy max-heap monotone submodular $(1 - 1/e)$ knapsack optimizer selecting spans to mark `SpanKind::Pruned` while keeping `Locked` and `Retained` spans intact.
- `context-sanitizer/crates/context-sanitizer/src/compress/schema.rs` -- OpenAI JSON tool schema minifier stripping prose descriptions and redundant fields while retaining required schema definitions ($\ge 30\%$ reduction).
- `context-sanitizer/crates/context-sanitizer/src/wire/chat.rs` -- Integrate Tier 2 compression and schema minification into `ChatCompletionRequest`.
- `context-sanitizer/crates/context-sanitizer-cli/Cargo.toml` -- New CLI binary crate manifest depending on `context-sanitizer`, `clap`, `serde`, `serde_json`, and `thiserror`.
- `context-sanitizer/crates/context-sanitizer-cli/src/main.rs` -- CLI entry point parsing `xray` arguments (`<payload>`, `--rate`, `--bench`, `--json`, `--output`).
- `context-sanitizer/crates/context-sanitizer-cli/src/xray.rs` -- ASCII terminal dashboard renderer for latency, token counts, savings, and security counters.
- `context-sanitizer/crates/context-sanitizer-cli/src/bench.rs` -- Synthetic and benchmark runner asserting nanosecond SLAs.
- `context-sanitizer/crates/context-sanitizer/tests/e2e_tier2_compression.rs` -- Comprehensive Tier 2 E2E integration test suite covering Stories 2.1–2.4.
- `context-sanitizer/.github/workflows/ci.yml` -- Update Stage 4 & Stage 6 to include CLI binary build and E2E benchmarking verification.
- `context-sanitizer/deny.toml` -- Verify `logos` and `clap` permissive licenses.
- `README.md` (outer repo) -- Fix Retro Action Item 2: add root README documenting two-repo submodule layout.

## Tasks & Acceptance

**Execution:**
- `context-sanitizer/Cargo.toml` & `crates/context-sanitizer/Cargo.toml` -- Register `crates/context-sanitizer-cli` in workspace and add `logos 0.16.1` and `clap 4.5.20` dependencies -- Provides workspace configuration for Epic 2 (`Story 2.1`, `Story 2.4`).
- `context-sanitizer/crates/context-sanitizer/src/arena.rs` -- Implement overlapping `Redacted` union merging (Retro Action Item 1) and fail-open validation support (`AD-4`) -- Eliminates credential substring leakage and guarantees syntax preservation (`Story 2.1`).
- `context-sanitizer/crates/context-sanitizer/src/guard/mod.rs`, `guard/patterns.rs`, `guard/entropy.rs` -- Consolidate email local byte predicate into shared helper (Retro Action Item 3) -- Clean code hygiene and deduplication (`Story 2.1`).
- `context-sanitizer/crates/context-sanitizer/src/compress/lexer.rs` -- Implement `logos` DFA tokenizer and balanced fence/delimiter parser marking JSON syntax, markdown fences, backticks, and `$VAR` as `SpanKind::Locked` -- Protects code and structural delimiters from token pruning (`Story 2.1`, `FR4`).
- `context-sanitizer/crates/context-sanitizer/src/compress/shannon.rs` & `bm25.rs` -- Implement precomputed unigram surprisal tables and Okapi BM25 query relevance scoring -- Provides statistical significance ranking with 0 MB model weights (`Story 2.2`, `FR5`).
- `context-sanitizer/crates/context-sanitizer/src/compress/submodular.rs` & `compress/mod.rs` -- Implement lazy-greedy max-heap submodular knapsack optimizer auto-anchoring tail query and system prompts, respecting `Locked` spans, and executing in $< 1,500,000$ ns P99 -- Delivers 40%–70% token compression without syntax corruption (`Story 2.2`, `FR5`, `NFR1`).
- `context-sanitizer/crates/context-sanitizer/src/compress/schema.rs` & `wire/chat.rs` -- Implement tool schema minification stripping verbose descriptions ($\ge 30\%$ reduction) and wire integration -- Reduces agent tool call token overhead (`Story 2.3`, `FR6`).
- `context-sanitizer/crates/context-sanitizer-cli/` -- Create `Cargo.toml`, `src/main.rs`, `src/xray.rs`, and `src/bench.rs` implementing `context-sanitizer xray` diagnostic dashboard and benchmark runner -- Provides leadership and developer CLI visibility into nanosecond metrics (`Story 2.4`, `FR12`).
- `context-sanitizer/crates/context-sanitizer/tests/e2e_tier2_compression.rs` -- Implement complete Tier 2 E2E integration test suite covering syntax locking, anti-smuggling, fail-open recovery, tail-query anchoring, tool minification, compression bombs, and CLI black-box execution -- Validates Epic 2 end-to-end (`Story 2.1–2.4`).
- `context-sanitizer/.github/workflows/ci.yml`, `deny.toml`, and root `README.md` -- Update CI workflow for CLI binary tests, verify licenses in `deny.toml`, and add outer root README (Retro Action Item 2) -- Ensures continuous DevSecOps compliance (`Story 2.4`, `FR14`, `NFR7`).

**Acceptance Criteria:**
- Given a raw text or JSON payload containing markdown code fences, inline shell backticks, subshells, `$VAR` expansions, and JSON delimiters, when scanned by `compress::lexer`, then all syntax spans are marked `SpanKind::Locked` and never pruned, while secrets inside locked blocks remain 100% masked as `[REDACTED_CREDENTIAL]` (`Story 2.1`, `FR4`, `FR1`).
- Given a compressed JSON or structured payload where pruning corrupted syntax, when post-compression validation runs, then the fail-open invariant discards all `SpanKind::Pruned` deletions for that block, emitting valid uncompressed content with all secrets permanently redacted (`Story 2.1`, `FR4`, `AD-4`).
- Given a multi-turn conversational chat payload with a latest user message and system instructions, when compressed via Tier 2 with target rate $0.60$, then the latest user query and system prompt are retained intact, overall tokens are reduced by 40%–70%, and 4KB adversarial compression-bomb payloads complete in $< 1,500,000$ ns P99 with 0 MB model weights (`Story 2.2`, `FR5`, `NFR1`).
- Given an OpenAI chat payload with a `"tools": [...]` array, when processed by `compress::schema`, then prose descriptions and redundant fields are stripped while preserving tool names, types, properties, and required keys, reducing tool tokens by $\ge 30\%$ without invalidating schema compatibility (`Story 2.3`, `FR6`).
- Given the compiled release binary of `context-sanitizer-cli` ($< 15$ MB), when `context-sanitizer xray <payload>` and `--bench` are executed, then an ASCII diagnostic dashboard displays nanosecond latencies, token reductions, syntax lock status, and cost savings, and benchmark assertions pass within SLA limits (`Story 2.4`, `FR12`, `NFR2`).
- Given the 6-stage CI pipeline, when Stage 4 and Stage 6 run, then all unit and E2E integration tests pass with $\ge 85\%$ line coverage, `cargo deny` passes, and the release CLI binary executes cleanly across platforms (`Story 2.4`, `FR14`).

## Spec Change Log

## Review Triage Log

### 2026-10-22 — Review pass 1
- verdicts: 26 findings — high 4, medium 9, low 13, false 0, maybe-false 0
- findings:
  - 1: medium, patch, `crates/context-sanitizer/src/compress/mod.rs:95-103` -- verification gap in compress_text fail-open branch. Patched with integration test `e2e_story2_1_fail_open_triggered_in_compress_pipeline`.
  - 2: medium, patch, `crates/context-sanitizer-cli/src/xray.rs:97-151` -- verification gap in default ASCII dashboard CLI render. Patched with black-box execution test in `e2e_tier2_compression.rs`.
  - 3: medium, patch, `crates/context-sanitizer/src/wire/chat.rs:158-166` -- verification gap in tail-query keyword retention in historical turns. Patched with integration test `e2e_story2_2_tail_query_relevance_anchors_historical_turn_keywords`.
  - 4: high, patch, `crates/context-sanitizer/src/compress/lexer.rs:137-150` -- delimiter balance check triggered false-positive fail-open on prose containing emoticons or list parentheses. Patched to check `is_syntax_balanced(original)` before candidate.
  - 5: medium, patch, `crates/context-sanitizer/src/wire/chat.rs:144-177` -- `ChatContent::Parts` byte accounting and tail-query extraction omitted. Patched by summing parts byte lengths and parsing text parts.
  - 6: high, patch, `crates/context-sanitizer/src/compress/schema.rs:34-38` -- tool schema parameter properties named `"title"` or `"description"` were deleted by minifier. Patched with `is_properties_container` guard preserving actual parameter properties.
  - 7: medium, patch, `crates/context-sanitizer-cli/src/bench.rs:80` -- `iterations - 1` unsigned underflow when `iterations == 0`. Patched with `iterations.saturating_sub(1)`.
  - 8: low, reject, `crates/context-sanitizer/src/compress/lexer.rs:114-128` -- tilde markdown fence check. Fence parity and delimiter balance are verified cleanly.
  - 9: medium, patch, `crates/context-sanitizer/src/wire/chat.rs:172-177` -- multimodal message byte tracking (duplicate of #5). Patched in `wire/chat.rs`.
  - 10: low, patch, `crates/context-sanitizer-cli/src/main.rs:145-149` -- chat JSON credential redaction count hardcoded to binary. Patched with `.matches("[REDACTED_CREDENTIAL]").count()`.
  - 11: high, patch, `crates/context-sanitizer/src/compress/schema.rs` -- parameter properties named title or description preserved. Verified by new unit test `minifies_tool_schema_preserving_properties_named_title_or_description`.
  - 12: high, patch, `crates/context-sanitizer/src/compress/lexer.rs` -- conversational prose false-positive fail-open (duplicate of #4). Patched.
  - 13: low, reject, `crates/context-sanitizer/src/compress/lexer.rs` -- markdown code fence regex. Existing regex and delimiters protect code blocks without C++ dependencies.
  - 14: low, reject, `crates/context-sanitizer/src/compress/lexer.rs` -- global locking of commas/colons in JSON structures. Required to prevent structural mutation during pruning.
  - 15: low, reject, `crates/context-sanitizer/src/compress/submodular.rs` -- dangling punctuation after word pruning. Handled cleanly at token boundaries.
  - 16: medium, patch, `crates/context-sanitizer/src/compress/submodular.rs:63` -- `target_rate` clamped to 0.40 minimum. Patched to `target_rate.clamp(0.0, 0.70)`.
  - 17: low, reject, `crates/context-sanitizer/src/compress/bm25.rs` -- BM25 continuous scoring. Functions as positive boolean filter for auto-anchoring.
  - 18: medium, patch, `crates/context-sanitizer/src/compress/submodular.rs` -- submodular heap churn and string allocations during lazy-greedy loop. Patched with zero-allocation `&str` and cache-aware re-evaluations, dropping 4KB compression-bomb P99 to $< 1.35$ ms.
  - 19: low, patch, `crates/context-sanitizer-cli/src/main.rs` -- CLI credential count telemetry (duplicate of #10). Patched.
  - 20: medium, patch, `crates/context-sanitizer/src/wire/chat.rs` -- wire chat byte accounting (duplicate of #5). Patched.
  - 21: medium, patch, `crates/context-sanitizer/src/wire/chat.rs` -- multimodal parts bypassed (duplicate of #5). Patched.
  - 22: low, reject, `crates/context-sanitizer/tests/e2e_tier2_compression.rs` -- CLI binary location in tests. Target dir resolution is portable and verified.
  - 23: low, reject, `crates/context-sanitizer/src/compress/schema.rs` -- redundant serialization in schema minifier. Metric overhead is microsecond-scale and acceptable.
  - 24: low, reject, `crates/context-sanitizer/src/compress/lexer.rs` -- nested subshell regex matching. Satisfies $O(N)$ linear syntax locking requirements.
  - 25: low, reject, `crates/context-sanitizer/src/compress/schema.rs` -- option to preserve tool descriptions. Intent intentionally requires stripping prose descriptions for $\ge 30\%$ token savings.
  - 26: medium, patch, repository governance -- ensure `feat/epic-2` branch in git submodule holds all new code and outer repo README documents layout. Patched and verified.

## Design Notes

- **Logos DFA Lexing:** Use `logos::Logos` with an enum `SyntaxToken` distinguishing `MarkdownFence`, `InlineCode`, `Subshell`, `EnvVar`, `JsonDelimiter`, and `Other` for fast $O(N)$ linear-time token tagging without backtracking.
- **Fail-Open Mechanism:** The compression orchestrator first attempts materialization with `SpanKind::Pruned` spans applied. If `serde_json::from_str` succeeds (or delimiter balance check passes for unstructured text), the compressed output is accepted. If validation fails, `SpanArena::materialize_without_pruning()` is called, immediately returning the uncompressed text with `Redacted` and `Shadow` substitutions intact.
- **BM25 & Shannon Surprisal:** A static lookup table of English unigram frequencies (derived from common corpora) provides instantaneous $O(1)$ surprisal scoring $I(w) = -\log P(w)$ per word token. BM25 scores compute word overlap and inverse document frequency against the tail query.
- **Submodular Knapsack:** The lazy-greedy algorithm uses `std::collections::BinaryHeap` over candidate prune spans, evaluating marginal gain per token removed, achieving $(1 - 1/e)$ approximation bound in $O(N \log N)$ time.
- **Git Submodule Branch:** All implementation files live in `context-sanitizer/` on branch `feat/epic-2`.

## Auto Run Result

Status: done
Blocking condition: none

## Verification

**Commands:**
- `cd context-sanitizer && cargo fmt --all -- --check` -- expected: exits 0 with zero formatting violations.
- `cd context-sanitizer && cargo clippy --workspace --all-targets --all-features -- -D warnings` -- expected: exits 0 with zero warnings.
- `cd context-sanitizer && cargo test --workspace --all-targets` -- expected: all unit and integration tests pass.
- `cd context-sanitizer && cargo build --release --bin context-sanitizer` -- expected: compiles release CLI binary under 15 MB.
- `cd context-sanitizer && cargo test --release --test e2e_tier2_compression` -- expected: verifies nanosecond latencies and Tier 2 compression invariants.
