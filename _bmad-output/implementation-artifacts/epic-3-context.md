# Epic 3 Context: Pluggable System One ML Threat Gatekeeper & Fault-Isolated Fallback (Tier 3 E2E)

<!-- Generated from planning artifacts. Regenerate with compile-epic-context if planning docs change. -->

## Goal

Platform security engineers and systems developers can extend the pipeline via decoupled `SemanticGuard` and `TokenCompressor` traits, block structural prompt-injection and markdown exfiltration attacks in `< 100,000 ns` via an $O(N)$ lexical fast-breaker, or compile with `--features ml` (`candle-transformers` with quantized `Llama-Prompt-Guard-2-22M` or `DeBERTa-v3-xsmall`) to detect semantic prompt injections across top-$K$ ($K \le 2$) surprisal-ranked 512-token overlapping windows in `~1,500,000–3,500,000 ns` median (`< 5,000,000 ns` hard `guard_timeout_ns` P99 ceiling) on CPU, with guaranteed zero-crash automatic fallback to Tier 2 statistical compression when model checkpoints are missing, corrupt, or exceed the execution deadline.

## Stories

- Story 3.1: Pluggable Trait Decoupling (`SemanticGuard` & `TokenCompressor`) & Zero-ML Default Build Isolation
- Story 3.2: Two-Stage Nanosecond Threat Gatekeeper (`< 100,000 ns` Fast-Breaker + `< 5,000,000 ns` Anti-Sponge Encoder)
- Story 3.3: Fault-Isolated Graceful Fallback to Tier 2, Offline Micro-Checkpoint Fixture & Tier 3 E2E CI Gate

## Requirements & Constraints

- **Pluggable Trait Decoupling:** Export thread-safe `pub trait SemanticGuard: Send + Sync` (with `async fn evaluate_threat(&self, text: &str) -> Result<f32, SanitizerError>`) and `pub trait TokenCompressor: Send + Sync` (with `fn compress(&self, payload: &str, target_rate: f32) -> Result<String, SanitizerError>`) from `crates/context-sanitizer/src/ml/traits.rs` so downstream consumers and custom integrations can provide alternative neural classifiers or compressors polymorphically through the pipeline builder without modifying Core.
- **Zero-ML Default Build Isolation:** Core library builds without `--features ml` must completely exclude `candle-core`, `candle-transformers`, `tokenizers`, PyTorch, and ONNX from the dependency tree and compiled binary, ensuring 0 MB of downloaded model weights and `< 15 MB` compiled binary footprint for default distributions.
- **Stage 3A-Fast Lexical Circuit Breaker:** Ingest payload text and scan for structural injection and exfiltration patterns (role/delimiter spoofing like `<|im_start|>system`, `[INST]`, `### System:`, prompt override phrases like `Ignore previous instructions`, markdown exfiltration `![...](https://...)`, and encoded secret smuggling) in $O(N)$ linear time (`< 100,000 ns` median / `~10–25 ns/byte`). When detected, short-circuit immediately with `Err(SanitizerError::InjectionBlocked)` prior to neural tensor operations.
- **Stage 3B-Neural Surprisal-Ranked Sliding-Window Inference:**
  - When `--features ml` is active, evaluate incoming text using quantized System One non-autoregressive encoder checkpoints (`meta-llama/Llama-Prompt-Guard-2-22M` or `MoritzLaurer/deberta-v3-xsmall-zeroshot-v1.1-all-33`).
  - Texts exceeding the 512-token encoder limit must be segmented into 512-token windows using an overlapping sliding stride (default 128-token overlap / 384-token stride) to prevent boundary-splitting evasion attacks.
  - To prevent Neural Sponge CPU exhaustion DoS attacks, rank candidate windows by Tier 2 Shannon surprisal and evaluate at most the top-$K$ ($K \le 2$) highest-surprisal windows.
  - Enforce Early-Exit Circuit Breaking: abort pipeline execution immediately on the first window exceeding the threat threshold (default `0.85`), returning `SanitizerError::InjectionBlocked`.
  - Enforce a strict execution deadline bounded by `guard_timeout_ns` (default `5,000,000 ns` / 5 ms P99 ceiling; median `~1,500,000–3,500,000 ns`).
- **Fault-Isolated Graceful Fallback:** If a model checkpoint file is missing, corrupt or truncated, has an invalid `tokenizer.json`, fails tensor execution on host CPU hardware, or exceeds `guard_timeout_ns`, the engine must emit a structured non-fatal warning log and automatically fall back to Tier 1 deterministic guard + Stage 3A-Fast lexical breaker + Tier 2 statistical compression without panicking, unwrapping, or crashing the host process.
- **CI DevSecOps & Offline Fixtures:**
  - Unit and integration tests must run completely offline without remote network downloads, utilizing a committed micro-SafeTensors + `tokenizer.json` test fixture (`< 500 KB`) located under `crates/context-sanitizer/tests/fixtures/ml/`.
  - Integration suite `crates/context-sanitizer/tests/e2e_tier3_ml_gatekeeper.rs` must verify offline: custom trait polymorphism, `< 100,000 ns` fast-breaker blocking, $>512$-token sliding-window detection, top-$K$ surprisal window bounding, Tier 1 redaction + Tier 3 threat blocking synergy, and missing/corrupt checkpoint graceful fallback.
  - Pre-build coverage gate: `cargo llvm-cov --workspace --all-targets --all-features --fail-under-lines 85` must pass before `--features ml` release builds.
- **Safety & Quality Standards:** Enforce `#![forbid(unsafe_code)]` and `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]` across all library modules.

## Technical Decisions

- **Feature Flag Architecture:** In `crates/context-sanitizer/Cargo.toml`, gate all neural dependencies under optional feature `ml = ["dep:candle-core", "dep:candle-transformers", "dep:tokenizers"]`. Core without `--features ml` remains pure Rust with zero ML crate dependencies.
- **Pinned Neural Stack Dependencies:**
  - `candle-core = "0.11.0"`
  - `candle-transformers = "0.11.0"`
  - `tokenizers = "0.20.0"`
- **Core ML Subsystem Layout (`crates/context-sanitizer/src/ml/`):**
  - `traits.rs`: Defines public interfaces `pub trait SemanticGuard: Send + Sync` (with `async fn evaluate_threat(&self, text: &str) -> Result<f32, SanitizerError>`) and `pub trait TokenCompressor: Send + Sync` (with `fn compress(&self, payload: &str, target_rate: f32) -> Result<String, SanitizerError>`).
  - `fast_breaker.rs`: High-speed $O(N)$ lexical scanner detecting structural delimiters, prompt override strings, and markdown exfiltration syntax.
  - `chunking.rs`: 512-token sliding-window chunker with 128-token overlap, calculating window Shannon surprisal via Tier 2 static tables to select top-$K$ ($K \le 2$) windows.
  - `candle.rs`: Candle encoder inference implementation loading SafeTensors checkpoints and running CPU tensor classification, gated under `#[cfg(feature = "ml")]`.
  - `mod.rs`: Tier 3 orchestrator executing Stage 3A-Fast, surprisal-ranked Stage 3B-Neural inference, deadline enforcement (`guard_timeout_ns`), and seamless fallback to Tier 2 on error or timeout.
- **Error Handling Invariants:** Extend `SanitizerError` with `InjectionBlocked { reason: String, score: f32 }`, `ModelLoadError(String)`, and `GuardTimeout(u64)`. Ensure all model initialization and execution paths return `Result` and never unwrap.
- **Pipeline Configuration:** Introduce configurable parameters on the pipeline builder: `threat_threshold: f32` (default `0.85`), `guard_timeout_ns: u64` (default `5_000_000 ns`), and `max_windows_k: usize` (default `2`).

## UX & Interaction Patterns

- **CLI Interaction (`context-sanitizer xray` with `--features ml`):**
  - `context-sanitizer xray <payload.json> --threat-threshold <f32>`: Configures the threat sensitivity threshold (default `0.85`).
  - Dashboard Diagnostic Reporting: Displays Tier 3 execution time (`ns` and `ns/byte`), evaluated window count ($K \le 2$), maximum observed threat score, and gatekeeper verdict (`PASS` vs `BLOCKED`).
  - Exit Codes: Returns non-zero exit code when an injection attack is intercepted and blocked, preventing malicious payload propagation in automated pipelines.

## Cross-Story Dependencies

- **Story 3.1 $\to$ Stories 3.2 & 3.3:** Story 3.1 establishes the trait contracts (`SemanticGuard`, `TokenCompressor`) and Cargo feature gate (`--features ml`), which are prerequisites for implementing the fast-breaker and neural classifiers (`Story 3.2`) and fallback handlers (`Story 3.3`).
- **Story 3.2 $\to$ Story 3.3:** Story 3.2 builds the two-stage threat detection engine (lexical fast-breaker and Candle sliding-window classifier), which Story 3.3 wraps with timeout watchdogs, fallback logic, and tests against offline micro-fixtures.
- **Epic 1 & Epic 2 $\to$ Epic 3:**
  - Epic 1 provides the base `Span` arena, `SanitizerError`, and wire ingestion pipeline.
  - Epic 2 provides the unigram Shannon self-information calculations (`compress/shannon.rs`) utilized by `chunking.rs` to rank high-surprisal windows, as well as the Tier 2 statistical compression engine that serves as the fallback target when ML inference is unavailable.
- **Epic 3 $\to$ Epics 4 & 5:**
  - Epic 4 exposes the 3-tier cascade and configurable threat thresholds to Python frameworks (`context-sanitizer-py`).
  - Epic 5 builds on the clean feature isolation to ensure WebAssembly packages compile pure Tier 1 and Tier 2 without pulling in native ML dependencies.
