---
status: final
stepsCompleted: [1, 2, 3, 4]
inputDocuments:
  - _bmad-output/planning-artifacts/prds/prd-context-sanitizer-2026-10-22/prd.md
  - _bmad-output/planning-artifacts/prds/prd-context-sanitizer-2026-10-22/addendum.md
  - _bmad-output/planning-artifacts/architecture/architecture-context-sanitizer-2026-10-22/ARCHITECTURE-SPINE.md
---

# context-sanitizer - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for context-sanitizer, decomposing the requirements from the PRD, UX Design if it exists, and Architecture requirements into implementable stories.

## Requirements Inventory

### Functional Requirements

FR1: Comprehensive Credential Interception & One-Way Redaction — Detect root cloud credentials, service keys (AWS `AKIA...`, GCP `AIza...` & service accounts, Azure, Kubernetes bearer tokens/TLS certs, Okta, Salesforce, GitHub `ghp_...`, GitLab, Slack `xoxb-`/`xoxp-`, PGP/SSH private key blocks), generalized key-value patterns (`*_KEY`, `*_SECRET`, `*_TOKEN`, `*_AUTH`, `*_PASS`, `*_PASSWORD`, `*_CREDENTIAL`, `*_APIKEY`, `*_PRIVATE_KEY` with value >= 16 chars), and standalone high-entropy Shannon randomness ($H \ge 4.5$, length $\ge 20$), replacing them irreversibly with `[REDACTED_CREDENTIAL]` without storing the raw secret in any vault or memory struct.
FR2: Operational PII Vaulting & Cryptographic Shadow Token Generation — Detect operational PII (email addresses, IPv4/IPv6 addresses, telephone numbers) and replace each unique entity within a session with a cryptographically isolated shadow token formatted as `<SHADOW:<TYPE>_<HEX6>>` (e.g., `<SHADOW:EMAIL_8f3a2c>`), ensuring deterministic reuse within the same session and non-deterministic isolation across concurrent sessions via an in-memory `VaultHandle`.
FR3: Local Tool Rehydration with Scope & Destination Whitelisting — Provide a thread-safe `.rehydrate(args, destination_tool)` method on `VaultHandle` that replaces shadow tokens in tool call argument strings with their original raw PII values only when `destination_tool` is registered and authorized on the session whitelist, rejecting unauthorized or unwhitelisted destinations with an error.
FR4: Syntax Token Locking & Fail-Open AST Invariant — Lexically analyze payloads using a pure-Rust tokenizer (`logos` + `serde_json`), marking structural JSON characters (`{`, `}`, `[`, `]`, `:`, `,`), key names, markdown code fences (```` ``` ````), inline shell backticks (`` `command` ``), subshell substitutions (`$(...)`), and environment variable expansions (`$VAR`, `${VAR}`) as immutable locked tokens; if a compressed chunk fails JSON syntax validation or structural integrity checks post-compression, fail open to emit the uncompressed text (with secrets still permanently masked) to guarantee 0% downstream JSON parse errors.
FR5: Tail-Query Auto-Anchoring & Statistical Context Pruning — Calculate token significance using static unigram Shannon self-information ($I(w) = -\log P(w)$) and query-anchored Okapi BM25 relevance scoring against the user's latest query, auto-anchoring the latest user message, system prompt directives, and active schema keys with infinite priority while pruning low-entropy filler tokens to the target compression ratio (`--rate`, achieving 40%–70% token reduction on conversational corpora within 1.5% task accuracy variance).
FR6: JSON Tool Schema Minification — Detect OpenAI-format tool definitions (`"tools": [...]`) and strip unnecessary whitespace, optional parameter descriptions, and redundant type annotations while preserving required fields and property keys, reducing tool schema token count by >= 30% without altering semantic schema compatibility.
FR7: Pluggable Trait Decoupling (`SemanticGuard` & `TokenCompressor`) — Define clean, decoupled Rust traits (`pub trait SemanticGuard` and `pub trait TokenCompressor`) in the core crate so downstream consumers can implement custom guardrails or compressors, and ensure compiling without `--features ml` completely excludes Candle, PyTorch, and ONNX dependencies from the binary.
FR8: Two-Stage Nanosecond Threat Gatekeeper (`< 100,000 ns` Fast-Breaker + `< 5,000,000 ns` Surprisal-Guided Encoder) — Block structural prompt injections, delimiter spoofing (`<|im_start|>`, `[INST]`), and markdown image exfiltration (`![...](https://...)`) in `< 100,000 ns` (`~10–25 ns/byte`) via $O(N)$ SIMD fast-breaking, and when compiled with `--features ml` (`meta-llama/Llama-Prompt-Guard-2-22M`, `MoritzLaurer/deberta-v3-xsmall-zeroshot-v1.1-all-33`), evaluate the top-$K$ ($K \le 2$) highest-surprisal 512-token overlapping windows within `~1,500,000–3,500,000 ns` median (`< 5,000,000 ns` hard `guard_timeout_ns` P99 ceiling) to prevent Neural Sponge DoS attacks, short-circuiting immediately when exceeding `--threat-threshold` (default `0.85`).
FR9: Graceful Fallback & Fault Isolation — If an ML model checkpoint file is missing, corrupt, or fails to execute on the host CPU architecture when `--features ml` is active, log a non-fatal warning and gracefully fall back to Tier 2 statistical pruning without panicking or crashing the host process.
FR10: OpenAI-Compatible Wire Ingestion & Payload Pass-Through — Ingest, sanitize/compress, and serialize structured chat message arrays (`POST /v1/chat/completions` and gateway alias `POST /v1/chat`) and legacy text completion payloads (`POST /v1/completions`), preserving exact OpenAI schema parity (`messages`, `role`, `content`, `tools`, `tool_calls`) across Rust, Python, and CLI with zero structural drift.
FR11: Zero-C Toolchain Python Wheel Distribution & Turnkey Agent Middleware — Distribute `context-sanitizer-py` as precompiled zero-C wheels via `maturin` (`manylinux_2_28`, `macos-universal2`, `windows-x64`) exposing `sanitize()`, `rehydrate()`, and `VaultHandle`, along with two-line turnkey middleware integrations (`LangChainSanitizer`, `CrewAISanitizer`, `AutoGenSanitizer`).
FR12: Context X-Ray Diagnostic & Benchmarking CLI (`context-sanitizer xray`) — Provide a standalone CLI (`context-sanitizer xray <payload> [--bench]`) outputting an ASCII diagnostic dashboard with nanosecond (`ns`) latency per tier (`std::time::Instant::elapsed().as_nanos()` and `ns/byte`), tokens before/after, percentage pruned, estimated dollar savings across LLM pricing tiers (GPT-4o, Claude 3.5 Sonnet), vaulted PII counts, syntax lock status, threat scores, and reproducible nanosecond benchmark runs (`MeetingBank`, `GSM8K`, Synthetic Credential & PII Corpus).
FR13: Universal WebAssembly / WASI Compilation for Tier 1 & Tier 2 — Compile the Tier 1 and Tier 2 core engine cleanly to `wasm32-unknown-unknown` and `wasm32-wasip1` targets via `wasm-pack`, packaged as `@context-sanitizer/wasm` for client-side execution in browsers, Electron apps, Node.js, and Cloudflare Workers.
FR14: Automated DevSecOps CI/CD Pipeline, Local `pre-commit` Framework & Pre/Post-Build Integration Gates — Enforce a local `.pre-commit-config.yaml` hook suite (firing `cargo fmt`, `cargo clippy -- -D warnings`, `ruff`, `gitleaks`, `cargo deny check`, and local unit/integration tests on `git commit`/`git push`) alongside a staged 6-step GitHub Actions pipeline (`.github/workflows/ci.yml` and `release.yml`): (1) Formatting & Linting (`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `ruff format --check`, `ruff check`), (2) Source Code SAST & Secret Scanning (`#![forbid(unsafe_code)]`, `deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)`, Semgrep/CodeQL, `gitleaks`), (3) Software Composition Analysis (SCA) & License Auditing (`cargo audit`, `cargo deny check advisories bans licenses sources` via `deny.toml`, `pip-audit`), (4) Pre-Build Unit Tests (`cargo test --lib --bins`), Source Integration Tests (`cargo test --test '*'`), and Code Coverage Gate (`cargo llvm-cov --workspace --fail-under-lines 85` and `pytest --cov-fail-under=85`), (5) Gated Artifact Compilation (`cargo build --release`, `maturin build --release`, `wasm-pack build`), and (6) Post-Build Black-Box Artifact E2E Integration Tests & Benchmarks (testing compiled release CLI binaries + `xray --bench`, installing built `.whl` artifacts in clean virtualenvs for `pytest` E2E runs, and testing built Wasm bundles in JS/WASI runtimes).

### NonFunctional Requirements

NFR1: Performance, Nanosecond Latency Budgets & Anti-DoS Complexity — Tier 1 deterministic scanning executes in strict $O(N)$ linear time (`~20,000–100,000 ns` median / `~5–25 ns/byte`, `< 500,000 ns` P99 ceiling even on adversarial ReDoS payloads) per 4KB payload; Tier 2 statistical compression executes in $O(N \log N)$ lazy-greedy time (`~300,000–400,000 ns` median / `~75–100 ns/byte`, `< 1,500,000 ns` P99 ceiling even on adversarial "compression bomb" payloads, `< 500,000 ns` combined L0+L0.5 median) per 4KB payload; Tier 3 threat gatekeeping executes its lexical fast-breaker in `< 100,000 ns` (`~10–25 ns/byte`) and bounds quantized encoder passes to `~1,500,000–3,500,000 ns` median (`< 5,000,000 ns` hard `guard_timeout_ns` P99 ceiling via top-$K$ surprisal window selection, blocking Neural Sponge CPU exhaustion) on standard `x86_64` and Apple Silicon CPU cores.
NFR2: Memory Footprint, Concurrency & Binary Size — Tier 1 and Tier 2 allocate zero persistent heap structures outside active sessions, require 0 MB of downloaded model weights by default, and keep default binary size `< 15 MB`. `VaultHandle` (`Arc<RwLock<VaultInner>>`) supports 10,000+ concurrent requests/sec without lock contention.
NFR3: Memory Safety & Code Invariants — The core crate (`context-sanitizer`) enforces `#![forbid(unsafe_code)]` across all parsing and token manipulation modules, and library code uses typed `thiserror` enums (`SanitizerError`) with zero panics (`unwrap`/`expect` forbidden on untrusted inputs).
NFR4: Platform Portability & Zero-C Build Ergonomics — Zero external C/C++ compiler toolchain dependencies (`gcc`, `clang`, MSVC not required for end-user wheel installation; no C-based `tree-sitter`). Compiles cleanly on Windows, macOS, Linux, and Wasm targets.
NFR5: Security & Privacy Guardrails — Split-Secret Policy enforces permanent one-way redaction for root credentials with no rehydration mechanism under any condition, while operational PII is isolated in ephemeral, in-memory, session-salted vaults (128-bit cryptographic salt) with zero disk or cloud retention.
NFR6: Accuracy & False-Positive Guardrails (Counter-Metrics) — Submodular knapsack budget optimization preserves benchmark reasoning accuracy (`MeetingBank`, `GSM8K`) within `1.5%` variance of baseline (do not over-prune past 70% at the expense of accuracy or JSON validity), and credential regex/entropy rules avoid false-positive redaction of benign code identifiers (e.g., `const KEY_COUNT = 5`).
NFR7: Licensing & Open-Source Hygiene — MIT License across all workspace crates; all third-party dependencies must be commercially permissive (MIT, Apache-2.0, or BSD-3-Clause) and explicitly documented in `README.md`.
NFR8: DevSecOps, SAST/SCA & Pre/Post-Build Test Integrity — All binary, wheel, and Wasm builds are strictly gated behind passing formatting (`cargo fmt`/`ruff`), linting (`clippy`/`ruff`), SAST (`#![forbid(unsafe_code)]`, panic-free lints, Semgrep/CodeQL, `gitleaks`), SCA (`cargo-audit`, `cargo-deny`, `pip-audit`), and pre-build unit + source integration test coverage gates (`cargo-llvm-cov` and `pytest-cov` >= 85% line coverage), followed by mandatory post-build black-box E2E integration tests against the compiled release binaries, Python `.whl` packages, and Wasm bundles across Linux, macOS, and Windows.

### Additional Requirements

- **Greenfield Structural Seed & Incremental Crate Activation (Impacts Epic 1 Story 1):** Initialize the Cargo workspace (`Cargo.toml` root with `.github/workflows/`, `.pre-commit-config.yaml`, `.gitleaks.toml`, `deny.toml`, and `crates/context-sanitizer` Core) targeting Rust Edition 2021 (MSRV 1.80+) with pinned stack versions (`aho-corasick 1.1.5`, `logos 0.16.1`, `serde 1.0.210`, `serde_json 1.0.128`, `thiserror 1.0.64`, `clap 4.5.20`, `pyo3 0.23.0`, `maturin 1.7.4`, `wasm-bindgen 0.2.95`, and optional `candle-core 0.11.0`, `candle-transformers 0.11.0`, `tokenizers 0.20.0`). Surface crates (`crates/context-sanitizer-cli` in Epic 2, `crates/context-sanitizer-py` in Epic 4, `wasm/` in Epic 5) are registered in the workspace incrementally as each epic is built so `--workspace` coverage gates ($\ge 85\%$) never fail on empty stub crates.
- **AD-1 (Span-Annotated Pipes-and-Filters Cascade):** Implement `arena.rs` with `Span { start, end, kind }` and `SpanKind { Locked, Redacted(&'static str), Shadow(String), Pruned, Retained }`. All pipeline stages operate as read-only passes over an immutable payload buffer and materialize the sanitized output in a single final allocation. Redaction spans hold absolute precedence over compression spans.
- **AD-2 (Unidirectional Hexagonal Boundaries):** Enforce strict unidirectional dependencies where `context-sanitizer-cli`, `context-sanitizer-py`, and `wasm` depend on `context-sanitizer` (Core), and Core has zero dependencies on surface adapters or C/C++ libraries.
- **AD-3 (Cryptographic Session Vault Architecture):** Implement `VaultHandle` wrapping `Arc<RwLock<VaultInner>>` with `forward`/`reverse` hash maps, `whitelist: HashSet<String>`, and per-session `salt: [u8; 16]` generating `<SHADOW:<TYPE>_<HEX6>>` tokens; drop deallocates session state immediately.
- **AD-4 (Fail-Open Materializer & Post-Compression JSON Validator):** In the materializer stage, verify structured/JSON outputs via `serde_json::from_str`; if validation fails, discard `Pruned` span removals and emit the original text with `Redacted` and `Shadow` spans still applied.
- **AD-5 (Conditional ML Compilation, Offline Micro-Fixtures & Submodular Knapsack Selection):** Implement `submodular.rs` for monotone submodular $(1 - 1/e)$ budgeted knapsack chunk selection in Tier 2, gate `candle.rs` + `chunking.rs` behind `--features ml` with automatic fallback to Tier 2 on load/inference errors, and commit a tiny deterministic micro-SafeTensors + `tokenizer.json` fixture (`< 500 KB`) for offline CI testing without flaky Hugging Face network downloads.
- **AD-6 (Continuous DevSecOps Verification, Two-Speed `pre-commit` Hooks & Pre/Post-Build Gates):** Seed `.pre-commit-config.yaml` (fast `< 3s` `pre-commit` hooks for `fmt`, `clippy`, `ruff`, `gitleaks`, `cargo-deny`; full `pre-push` hooks for `cargo-audit`, unit + integration tests, and `cargo-llvm-cov`), `.gitleaks.toml` (allowlisting synthetic canary fixtures in `tests/fixtures/` and `benches/`), `.github/workflows/ci.yml`, and `deny.toml` in Epic 1. Enforce the 6-stage CI pipeline — (1) Formatting & Linting, (2) SAST & Secret Scanning, (3) SCA, (4) Pre-Build Unit Tests + In-Process Source Integration Tests + $\ge 85\%$ Coverage Gate (`cargo-llvm-cov`), (5) Gated Artifact Compilation, and (6) Post-Build Black-Box Artifact E2E Integration Tests — and feature-gate `pyo3/extension-module` in Epic 4 so `cargo test --workspace` links cleanly across all OS targets.

### UX Design Requirements

None — `context-sanitizer` is a headless systems library, CLI, Python wheel, and WebAssembly module with no graphical user interface (terminal ASCII dashboard requirements for `context-sanitizer xray` are captured in FR12).

### FR Coverage Map

FR1: Epic 1 - Deterministic credential interception & irreversible `[REDACTED_CREDENTIAL]` masking (vendor signatures, `*_KEY`/`*_SECRET` heuristics, Shannon entropy)
FR2: Epic 1 - Session-scoped operational PII detection & `<SHADOW:<TYPE>_<HEX6>>` token generation in `VaultHandle`
FR3: Epic 1 - Thread-safe `.rehydrate(args, destination)` with registered tool destination whitelisting
FR4: Epic 2 - Pure-Rust lexical syntax token locking (`logos` + `serde_json`) & Fail-Open Syntax Invariant
FR5: Epic 2 - Tail-query auto-anchoring & statistical context pruning (Shannon self-information + Okapi BM25 + submodular knapsack)
FR6: Epic 2 - OpenAI JSON `"tools"` schema minification (>= 30% token reduction)
FR7: Epic 3 - Pluggable `SemanticGuard` and `TokenCompressor` traits with zero-ML default build isolation
FR8: Epic 3 - System One encoder threat scoring (`candle-transformers`) with 512-token sliding window & threshold short-circuiting
FR9: Epic 3 - Fault-isolated graceful fallback from Tier 3 ML to Tier 2 statistical compression on missing/corrupt checkpoints
FR10: Epic 1 - OpenAI-compatible wire ingestion & round-trip serialization (`/v1/chat/completions`, `/v1/chat`, `/v1/completions`)
FR11: Epic 4 - Zero-C precompiled Python wheels (`context-sanitizer-py` via `maturin`) & turnkey middleware (`LangChainSanitizer`, `CrewAISanitizer`, `AutoGenSanitizer`)
FR12: Epic 2 - Context X-Ray diagnostic & benchmarking CLI (`context-sanitizer xray <payload> [--bench]`)
FR13: Epic 5 - Universal WebAssembly / WASI package (`@context-sanitizer/wasm` for `wasm32-unknown-unknown` and `wasm32-wasip1`)
FR14: Epic 1 (Local `.pre-commit-config.yaml` hooks + 6-stage DevSecOps CI spine: Formatting, Linting, SAST, SCA, Pre-Build Unit + Integration Tests + >=85% `cargo-llvm-cov` Coverage Gate $\to$ Release Build $\to$ Post-Build E2E Integration Tests) + extended in Epics 2–5 (Post-build CLI binary E2E + `xray --bench` gate, `--features ml` pre/post-build E2E gate, `ruff` pre-commit + `maturin` wheel build + clean-venv `pytest` E2E gate, and `wasm-pack` build + JS/WASI artifact E2E gate)

## Epic List

### Epic 1: Deterministic Boundary Guard, Split-Secret Vault, OpenAI Wire Pipeline & DevSecOps CI Foundation (Tier 1 E2E)
AI systems engineers can pass raw strings or OpenAI-compatible payloads (`/v1/chat/completions`, `/v1/chat`, `/v1/completions`) through the pure-Rust span-annotated pipeline to permanently redact root credentials (`[REDACTED_CREDENTIAL]`), vault operational PII into session-isolated `<SHADOW:<TYPE>_<HEX6>>` tokens in `~20,000–100,000 ns` median (`< 500,000 ns` P99), and safely rehydrate authorized local tool calls via `VaultHandle::rehydrate()`, backed from commit #1 by an automated DevSecOps CI pipeline (`fmt`, `clippy`, `cargo-audit`, `cargo-deny`, `gitleaks`, cross-platform E2E tests).
**FRs covered:** FR1, FR2, FR3, FR10, FR14 (Foundation)
**E2E Integration Test Target:** Ingest multi-turn OpenAI chat/completion JSON payloads containing synthetic cloud secrets and customer PII $\to$ verify single-allocation materialization, irreversible credential masking, deterministic in-session shadow tokens, cross-session cryptographic isolation, authorized vs. unauthorized tool argument rehydration, and green execution of the DevSecOps CI pipeline (`ci.yml` + `deny.toml`).

### Epic 2: Zero-Weight Syntax-Safe Statistical Compression & Context X-Ray CLI (Tier 2 + CLI E2E)
Developers and engineering leaders can compress multi-turn conversational contexts by 40%–70% and OpenAI JSON tool schemas by $\ge 30\%$ in `~300,000–400,000 ns` median (`< 1,500,000 ns` P99 even on adversarial compression-bomb inputs) with 0 MB of model weights, guaranteed 0% JSON/code syntax corruption via lexical token locking and the Fail-Open Syntax Invariant, and audit real-time nanosecond (`ns`) latency, token savings, and benchmark corpora via `context-sanitizer xray <payload> [--bench]` (including automated CI benchmark regression gates).
**FRs covered:** FR4, FR5, FR6, FR12, FR14 (CLI & Benchmark CI gate)
**E2E Integration Test Target:** Execute the core pipeline and compiled `context-sanitizer xray` CLI binary against OpenAI payloads containing code blocks, shell commands, and `"tools"` schemas as well as `MeetingBank`/`GSM8K` benchmark fixtures $\to$ verify locked syntax preservation, fail-open recovery on malformed chunks, 40%–70% token compression, $\ge 30\%$ schema minification, and terminal nanosecond diagnostic/benchmark reporting in CI.

### Epic 3: Pluggable System One ML Threat Gatekeeper & Fault-Isolated Fallback (Tier 3 E2E)
Platform security engineers can extend the pipeline via decoupled `SemanticGuard` and `TokenCompressor` traits, block structural prompt-injection and exfiltration patterns in `< 100,000 ns`, or enable `--features ml` (`candle-transformers` with quantized `Llama-Prompt-Guard-2-22M` / `DeBERTa-v3-xsmall`) to detect and short-circuit indirect prompt injections across top-$K$ ($K \le 2$) surprisal-ranked 512-token overlapping windows in `~1,500,000–3,500,000 ns` median (`< 5,000,000 ns` hard `guard_timeout_ns` P99) on CPU, with automatic zero-crash fallback to Tier 2 when model weights are unavailable, corrupt, or exceed the nanosecond deadline.
**FRs covered:** FR7, FR8, FR9, FR14 (`--features ml` CI gate)
**E2E Integration Test Target:** Run full 3-tier cascade integration tests (both default and `--features ml` in CI) covering custom trait implementations, $>512$-token sliding-window jailbreak/injection payloads that trigger threshold short-circuiting (`SanitizerError` / `InjectionBlocked`), and missing/corrupt model weight fixtures verifying non-fatal warning logs and seamless fallback to Tier 2 output.

### Epic 4: Zero-C Python Wheel & Turnkey Agent Framework Middleware (Python E2E)
Python AI engineers can install `context-sanitizer-py` without a C/C++ compiler toolchain and protect existing OpenAI, LangChain, CrewAI, and AutoGen agent pipelines in two lines of code with full access to `sanitize()`, `VaultHandle`, and authorized tool `.rehydrate()`, verified via automated `maturin` wheel CI builds across Linux, macOS, and Windows.
**FRs covered:** FR11, FR14 (Maturin Wheel & Pytest CI matrix)
**E2E Integration Test Target:** Build the `context-sanitizer-py` wheel via `maturin` and execute a `pytest` end-to-end multi-turn agent workflow suite verifying `cs.sanitize()`, `vault.rehydrate()`, `LangChainSanitizer`, `CrewAISanitizer`, and `AutoGenSanitizer` round-trip execution and GIL-free concurrency in CI.

### Epic 5: Universal WebAssembly, Edge Client Sanitization & Multi-Target Release Pipeline (Wasm + Release E2E)
Frontend and edge engineers can deploy `@context-sanitizer/wasm` (`wasm32-unknown-unknown` and `wasm32-wasip1`) inside browser extensions, Electron apps, Node.js, and Cloudflare Workers to perform client-side Tier 1 credential/PII sanitization and Tier 2 statistical compression before payloads ever leave the client runtime, complete with the final multi-target DevSecOps release and provenance pipeline (`release.yml`).
**FRs covered:** FR13, FR14 (Wasm CI & Multi-Target Release Automation)
**E2E Integration Test Target:** Compile the `wasm/` package via `wasm-pack` / WASI targets and run end-to-end JS/Wasm integration tests verifying client-side OpenAI payload sanitization, shadow token vaulting/rehydration, statistical compression, and full release pipeline validation.

## Epic 1: Deterministic Boundary Guard, Split-Secret Vault, OpenAI Wire Pipeline & DevSecOps CI Foundation (Tier 1 E2E)

AI systems engineers can pass raw strings or OpenAI-compatible payloads (`/v1/chat/completions`, `/v1/chat`, `/v1/completions`) through the pure-Rust span-annotated pipeline to permanently redact root credentials (`[REDACTED_CREDENTIAL]`), vault operational PII into session-isolated `<SHADOW:<TYPE>_<HEX6>>` tokens in `~20,000–100,000 ns` median (`< 500,000 ns` P99), and safely rehydrate authorized local tool calls via `VaultHandle::rehydrate()`, backed from commit #1 by the two-speed local `pre-commit` framework and 6-stage DevSecOps CI pipeline.

### Story 1.1: Workspace Foundation, Span Arena & Two-Speed DevSecOps CI/Pre-Commit Spine

As an AI Systems Engineer and Contributor,
I want a pure-Rust Cargo workspace initialized with the zero-copy `Span` arena materializer, two-speed `.pre-commit-config.yaml` hooks, and the 6-stage GitHub Actions DevSecOps pipeline,
So that every commit and pull request enforces formatting, linting, SAST, SCA, unit/integration tests, $\ge 85\%$ code coverage before build, and post-build verification over a memory-safe span pipeline foundation.

**Acceptance Criteria:**

**Given** a fresh clone of the repository on Rust Edition 2021 (MSRV 1.80+)
**When** the workspace and core crate (`crates/context-sanitizer`) are inspected and compiled
**Then** `crates/context-sanitizer/src/lib.rs` enforces `#![forbid(unsafe_code)]` and `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`, defines typed error enums (`SanitizerError`) via `thiserror`, and includes MIT license metadata (`FR14`, `NFR3`, `NFR7`, `AD-2`)
**And** `crates/context-sanitizer/src/arena.rs` implements `Span { start, end, kind }` and `SpanKind { Locked, Redacted(&'static str), Shadow(String), Pruned, Retained }` with a single-allocation materializer that resolves overlapping spans by enforcing absolute precedence of `Redacted` and `Shadow` spans over `Locked`, `Pruned`, and `Retained` spans without corrupting multi-byte UTF-8 character boundaries (`AD-1`).

**Given** a developer stages or pushes commits locally with `pre-commit` installed
**When** `git commit` or `git push` is triggered
**Then** `.pre-commit-config.yaml` runs fast `< 3s` `pre-commit` hooks (`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `gitleaks` using `.gitleaks.toml` allowlisting synthetic fixtures in `**/tests/fixtures/**` and `benches/**`, and `cargo deny check licenses bans`)
**And** the `pre-push` hook executes `cargo audit`, unit tests (`cargo test --workspace --lib --bins`), source integration tests (`cargo test --workspace --test '*'`), and `cargo llvm-cov --workspace --all-targets --fail-under-lines 85` (`FR14`, `AD-6`).

**Given** a pull request or push to the repository
**When** `.github/workflows/ci.yml` executes across Linux, macOS, and Windows
**Then** CI enforces in strict sequence: Stage 1 (Formatting & Linting), Stage 2 (SAST via Clippy AST lints, Semgrep/CodeQL, and `gitleaks`), Stage 3 (SCA via `cargo audit` and `cargo deny check advisories bans licenses sources` using `deny.toml`), Stage 4 (Unit Tests $\to$ Source Integration Tests $\to$ `cargo llvm-cov --workspace --all-targets --fail-under-lines 85` coverage gate), Stage 5 (Gated `cargo build --release`), and Stage 6 (Post-Build release artifact verification) (`FR14`, `NFR8`, `AD-6`).

### Story 1.2: Tier 1 Comprehensive Credential Interception, Irreversible Redaction & Anti-ReDoS Guarantee

As a Platform Security Engineer,
I want the Tier 1 deterministic guard to scan input buffers synchronously in $O(N)$ linear time (`~20,000–100,000 ns` median / `~5–25 ns/byte`, `< 500,000 ns` P99 per 4KB payload) using SIMD pattern matching, key-value suffix heuristics, and Shannon entropy scoring to replace root credentials with `[REDACTED_CREDENTIAL]`,
So that cloud keys, API tokens, and high-entropy secrets are permanently masked before any network dispatch and attackers cannot trigger ReDoS CPU exhaustion.

**Acceptance Criteria:**

**Given** an input payload containing known vendor credential signatures (AWS `AKIA...`, GCP `AIza...` and service account private keys, Azure storage/tenant secrets, Kubernetes bearer tokens and TLS certs, Okta `00...`, Salesforce, GitHub `ghp_...`, GitLab, Slack `xoxb-`/`xoxp-`, and PGP/SSH `-----BEGIN ... PRIVATE KEY-----` blocks)
**When** the Tier 1 scanner (`crates/context-sanitizer/src/guard/patterns.rs`) executes synchronously over the immutable buffer
**Then** it emits `SpanKind::Redacted("[REDACTED_CREDENTIAL]")` for every matched secret using `aho-corasick` SIMD DFA automaton matching (`FR1`, `AD-1`)
**And** the materialized output contains `[REDACTED_CREDENTIAL]` with zero occurrences of the raw secret bytes, and the redacted secret is never stored in any vault, struct, or log (`FR1`, `NFR5`).

**Given** an input payload containing generalized variable assignments or JSON/YAML key-value pairs matching case-insensitive suffixes (`*_KEY`, `*_SECRET`, `*_TOKEN`, `*_AUTH`, `*_PASS`, `*_PASSWORD`, `*_CREDENTIAL`, `*_APIKEY`, `*_PRIVATE_KEY`) or standalone high-entropy strings
**When** `guard/patterns.rs` and `guard/entropy.rs` evaluate the tokens
**Then** any matched key-value assignment whose value is $\ge 16$ characters, and any standalone unanchored token with length $\ge 20$ and Shannon entropy $H \ge 4.5$, is masked as `[REDACTED_CREDENTIAL]` (`FR1`)
**And** benign code identifiers or short values (such as `const KEY_COUNT = 5` or `token_limit = 4096`) are not flagged, preserving a 0% false-positive rate on standard code identifiers (`NFR6`).

**Given** both standard 4KB credential payloads and adversarial ReDoS payloads (e.g., repeated near-match prefixes `AKIA0000...` and nested `*_SECRET=` patterns)
**When** the Tier 1 guard pass is benchmarked on standard `x86_64` or Apple Silicon CPU hardware
**Then** the scanner executes strictly in $O(N)$ linear time with zero backtracking regexes, completing in `< 500,000 ns` P99 (`~20,000–100,000 ns` median / `~5–25 ns/byte`) with zero persistent heap allocations outside the call scope (`NFR1`, `NFR2`).

### Story 1.3: Operational PII Detection, Cryptographic Session Shadow Vault & Whitelisted Rehydration

As an AI Application Developer,
I want operational PII (emails, IPv4/IPv6 addresses, telephone numbers) automatically vaulted into session-scoped `<SHADOW:<TYPE>_<HEX6>>` tokens with authorized local tool rehydration via `VaultHandle::rehydrate()`,
So that raw customer PII never reaches external LLM providers while authorized internal tools can still execute seamlessly and unauthorized webhooks are blocked.

**Acceptance Criteria:**

**Given** an input payload containing email addresses, IPv4/IPv6 addresses, and telephone numbers within a single sanitization session
**When** the Tier 1 PII scanner and `VaultHandle` (`crates/context-sanitizer/src/guard/vault.rs`) process the payload
**Then** each unique PII entity is replaced with `<SHADOW:<TYPE>_<HEX6>>` (e.g., `<SHADOW:EMAIL_8f3a2c>`, `<SHADOW:IP_4d91b0>`, `<SHADOW:PHONE_2a1b9e>`) derived using the session's 128-bit cryptographic random salt (`[u8; 16]`) (`FR2`, `AD-3`)
**And** identical PII strings within the same session deterministically map to the exact same shadow token, while two concurrent sessions processing the identical PII string generate distinct, non-deterministic hex tokens (`FR2`, `NFR5`).

**Given** a `VaultHandle` containing vaulted PII mappings and a configured whitelist of authorized tool destinations (e.g., `["stripe_lookup", "postgres_query"]`)
**When** `vault.rehydrate(tool_args, "stripe_lookup")` is called on a tool argument string containing `<SHADOW:EMAIL_8f3a2c>`
**Then** the method returns the string with `<SHADOW:EMAIL_8f3a2c>` replaced by the original raw email address (`FR3`, `AD-3`)
**And** when `vault.rehydrate(tool_args, "https://evil.com/webhook")` or any unregistered destination is called, rehydration is rejected with `Err(SanitizerError::UnauthorizedRehydration)` and zero raw PII is returned (`FR3`, `AD-3`).

**Given** a multi-threaded runtime executing 10,000+ concurrent sanitization and rehydration operations per second
**When** `VaultHandle` (`Arc<RwLock<VaultInner>>`) is accessed concurrently and dropped at session end
**Then** all operations complete safely without data races or panics, and dropping the final `VaultHandle` reference immediately deallocates all session PII mappings from memory with zero disk persistence (`NFR2`, `NFR5`, `AD-3`).

### Story 1.4: OpenAI-Compatible Wire Ingestion, Tier 1 E2E Integration Suite & CI Gate Verification

As an AI Systems Engineer,
I want to ingest and serialize structured OpenAI `/v1/chat/completions`, `/v1/chat`, and `/v1/completions` JSON payloads end-to-end through the Tier 1 guard and shadow vault,
So that I can drop `context-sanitizer` into standard OpenAI-compatible pipelines and verify the complete Tier 1 workflow via automated pre-build and post-build E2E integration tests.

**Acceptance Criteria:**

**Given** a valid OpenAI JSON payload targeting `POST /v1/chat/completions` (or gateway alias `POST /v1/chat` with `messages`, `role`, `content`, `tools`, `tool_calls`) or `POST /v1/completions` (raw `prompt` string)
**When** `crates/context-sanitizer/src/wire/{chat.rs, completion.rs}` deserializes, sanitizes via Tier 1, and re-serializes the payload
**Then** all message contents and prompts have root credentials replaced with `[REDACTED_CREDENTIAL]` and operational PII replaced with `<SHADOW:<TYPE>_<HEX6>>` while preserving 100% OpenAI JSON schema parity and unknown pass-through fields (`FR10`).

**Given** the Epic 1 end-to-end integration test suite (`crates/context-sanitizer/tests/e2e_tier1_pipeline.rs`)
**When** Stage 4 (Pre-Build Unit $\to$ Integration $\to$ Coverage Gate) and Stage 6 (Post-Build verification) execute locally and in CI
**Then** the E2E suite verifies a full multi-turn agent workflow: (1) ingesting a multi-turn OpenAI chat payload containing synthetic AWS/GitHub credentials, adversarial ReDoS near-match strings, benign code variables (`const KEY_COUNT = 5`), and repeated customer emails/IPs/phones, (2) asserting `< 500,000 ns` P99 per 4KB scan latency (`~5–25 ns/byte` median), irreversible secret redaction, in-session shadow token consistency, and cross-session salt isolation, and (3) simulating an LLM tool call response where `vault.rehydrate()` succeeds for an authorized internal tool and returns `SanitizerError::UnauthorizedRehydration` for an unwhitelisted destination (`FR1`, `FR2`, `FR3`, `FR10`, `FR14`)
**And** `cargo llvm-cov --workspace --all-targets --fail-under-lines 85` passes with $\ge 85\%$ line coverage before `cargo build --release` completes (`FR14`, `NFR8`).

## Epic 2: Zero-Weight Syntax-Safe Statistical Compression & Context X-Ray CLI (Tier 2 + CLI E2E)

Developers and engineering leaders can compress multi-turn conversational contexts by 40%–70% and OpenAI JSON tool schemas by $\ge 30\%$ in `~300,000–400,000 ns` median (`< 1,500,000 ns` P99 even on adversarial compression-bomb payloads; `< 500,000 ns` combined L0+L0.5 median) with 0 MB of model weights, guaranteed 0% JSON/code syntax corruption via lexical token locking and the Fail-Open Syntax Invariant, and audit real-time nanosecond (`ns`) latency, token savings, and benchmark corpora via `context-sanitizer xray <payload> [--bench]`.

### Story 2.1: Pure-Rust Lexical Syntax Token Locking, Anti-Smuggling Precedence & Fail-Open AST Invariant

As an AI Application Developer,
I want the Tier 2 engine to lexically lock all structural JSON delimiters, keys, markdown code fences, inline shell commands, subshells, and environment variables against pruning while preserving Tier 1 redaction precedence—and automatically fail open if post-compression validation fails,
So that statistical token pruning never corrupts code blocks, shell syntax, or downstream OpenAI JSON schemas (0% JSON parse errors) and attackers cannot smuggle secrets inside locked syntax blocks.

**Acceptance Criteria:**

**Given** an input payload containing structural JSON characters (`{`, `}`, `[`, `]`, `:`, `,`), JSON object keys, markdown code fences (```` ``` ````), inline shell backticks (`` `command` ``), subshell substitutions (`$(...)`), and environment variable expansions (`$VAR`, `${VAR}`)
**When** the pure-Rust lexical scanner (`crates/context-sanitizer/src/compress/lexer.rs` using `logos` DFA + `serde_json` with bounded nesting depth) scans the buffer
**Then** it marks every structural and code/shell syntax span with `SpanKind::Locked` in $O(N)$ linear time with zero C/C++ `tree-sitter` dependencies (`FR4`, `NFR4`, `AD-1`, `AD-4`)
**And** downstream statistical pruning passes are strictly prevented from marking any `SpanKind::Locked` byte range as `SpanKind::Pruned`, while Tier 1 `SpanKind::Redacted` and `SpanKind::Shadow` spans strictly override `SpanKind::Locked` so secrets or PII embedded inside markdown code fences, shell commands, or JSON keys are still 100% redacted or vaulted (blocking syntax-lock smuggling) (`FR1`, `FR2`, `FR4`, `AD-1`).

**Given** a compressed payload chunk produced by the materializer
**When** the post-compression validator checks JSON and structural integrity (`serde_json::from_str` for structured payloads and balanced fence/delimiter checks)
**Then** if any validation check fails, the engine triggers the **Fail-Open Syntax Invariant**: it discards all `SpanKind::Pruned` removals for that block and emits the original uncompressed text while keeping all `SpanKind::Redacted` and `SpanKind::Shadow` replacements permanently applied (`FR4`, `AD-1`, `AD-4`)
**And** downstream JSON deserialization succeeds 100% of the time (0% JSON parse errors) (`FR4`).

### Story 2.2: Tail-Query Auto-Anchoring, Statistical Context Pruning & Anti-Compression-Bomb Knapsack Selection

As an AI Systems Engineer,
I want Tier 2 to prune low-entropy conversational filler in `~300,000–400,000 ns` median (`< 1,500,000 ns` P99 ceiling per 4KB payload, with 0 MB of downloaded model weights) using static unigram Shannon self-information, query-anchored Okapi BM25, and $O(N \log N)$ lazy-greedy submodular knapsack selection while auto-anchoring the latest user query and system instructions,
So that multi-turn prompt token volume drops by 40%–70% without degrading model reasoning accuracy (`<= 1.5%` variance) or allowing attackers to trigger quadratic CPU exhaustion via compression bombs.

**Acceptance Criteria:**

**Given** a multi-turn OpenAI chat payload or text prompt and a target compression ratio (e.g., `rate = 0.60`)
**When** the Tier 2 statistical compressor (`crates/context-sanitizer/src/compress/{shannon.rs, bm25.rs, submodular.rs}`) evaluates unlocked spans
**Then** the latest user message, system prompt directives (`role: "system"`), and active schema keys are assigned infinite priority (auto-anchored as `SpanKind::Retained`) and are never pruned (`FR5`)
**And** historical conversational turns are scored by combining static unigram Shannon self-information ($I(w) = -\log P(w)$) and query-anchored Okapi BM25 relevance against the user's latest query, using $O(N \log N)$ lazy-greedy max-heap monotone submodular $(1 - 1/e)$ knapsack selection to mark low-significance filler tokens as `SpanKind::Pruned` until the target compression ratio is met (`FR5`, `NFR1`, `NFR6`, `AD-1`, `AD-5`).

**Given** both standard 4KB multi-turn conversational payloads and adversarial 4KB "compression bomb" payloads (e.g., 2,000+ single-character tokens or deeply nested JSON structures)
**When** Tier 2 statistical pruning executes on standard `x86_64` or Apple Silicon CPU hardware
**Then** it completes in `~300,000–400,000 ns` median (`~75–100 ns/byte`, `< 500,000 ns` combined L0+L0.5 median) and strictly `< 1,500,000 ns` P99 on adversarial inputs, requiring 0 MB of external model weights and zero persistent heap allocations outside the session (`FR5`, `NFR1`, `NFR2`)
**And** across standard conversational and reasoning benchmark fixtures (`MeetingBank`, `GSM8K`), token count is reduced by 40%–70% without over-pruning beyond 70% or exceeding the `1.5%` reasoning accuracy variance guardrail (`FR5`, `NFR6`).

### Story 2.3: OpenAI JSON Tool Schema Minification

As an Agentic Framework Developer,
I want the Tier 2 engine to automatically detect and minify bloated OpenAI `"tools": [...]` JSON schema definitions by stripping redundant whitespace, optional descriptions, and verbose annotations while preserving required fields and property keys,
So that tool-heavy agent prompts save $\ge 30\%$ on tool schema tokens without breaking LLM function calling.

**Acceptance Criteria:**

**Given** an OpenAI chat payload containing a `"tools": [...]` array of function/tool JSON Schema definitions
**When** `crates/context-sanitizer/src/compress/schema.rs` processes the tool definitions
**Then** it strips insignificant whitespace, optional parameter prose descriptions, and redundant metadata annotations while strictly preserving tool `name`, `parameters.type`, `properties` keys, parameter types, `enum` values, and `required` arrays (`FR6`, `AD-4`)
**And** the resulting `"tools"` token count is reduced by $\ge 30\%$ while remaining 100% schema-valid for OpenAI, Anthropic, and local LLM tool-calling parsers (`FR6`, `AD-4`).

### Story 2.4: Context X-Ray Diagnostic & Benchmarking CLI (`context-sanitizer xray`) & Tier 2 Post-Build E2E Suite

As an Engineering Leader (Persona: Elena, `UJ-3`),
I want a standalone `context-sanitizer xray <payload> [--rate <f32>] [--bench]` CLI binary that visualizes per-tier nanosecond (`ns`) latency, token reduction, syntax locks, vaulted PII, and estimated dollar savings—and runs deterministic nanosecond benchmark and adversarial corpora in CI,
So that I can audit compression and security claims locally in seconds and verify the compiled CLI binary in post-build E2E integration tests.

**Acceptance Criteria:**

**Given** the `crates/context-sanitizer-cli` crate registered in the Cargo workspace (`src/{main.rs, xray.rs, bench.rs}`) and Criterion benchmarks in `benches/pipeline_throughput.rs`
**When** a user or CI job runs `context-sanitizer xray payload.json --rate 0.50`
**Then** the terminal renders an ASCII diagnostic dashboard displaying: Tier 1 and Tier 2 execution latency in nanoseconds (`ns` via `std::time::Instant::elapsed().as_nanos()` and `ns/byte`), tokens before and after compression, percentage pruned, count of redacted root credentials and vaulted PII shadow tokens, syntax lock preservation status (100% locked tokens intact), threat score, and estimated dollar savings across major LLM pricing tiers (GPT-4o, Claude 3.5 Sonnet) (`FR12`)
**And** the compiled default release binary size is `< 15 MB` with zero downloaded model weights (`NFR2`).

**Given** a user or CI job runs `context-sanitizer xray payload.json --bench`
**When** the benchmark harness executes against local deterministic test corpora (`MeetingBank` multi-turn compression fixtures, `GSM8K` reasoning preservation fixtures, Synthetic Credential & PII Corpus, and adversarial ReDoS / Compression-Bomb payloads)
**Then** it reports throughput (`ns/byte`), P50/P99 nanosecond latency (`< 500,000 ns` P99 Tier 1, `< 1,500,000 ns` P99 Tier 2, `< 500,000 ns` combined L0+L0.5 median per 4KB), compression ratio (`40%–70%`), tool schema reduction ($\ge 30\%$), and `0%` JSON parse errors, exiting with non-zero status if any nanosecond SLA budget regresses (`FR12`, `NFR1`, `NFR6`).

**Given** the 6-stage DevSecOps CI pipeline (`.github/workflows/ci.yml`) and `.pre-commit-config.yaml`
**When** Epic 2 is validated in Stage 4 (Pre-Build) and Stage 6 (Post-Build)
**Then** Stage 4 executes all workspace unit tests and in-process Tier 1 + Tier 2 integration tests (`crates/context-sanitizer/tests/e2e_tier2_compression.rs`, including syntax-lock secret smuggling and fail-open recovery tests) enforcing `cargo llvm-cov --workspace --all-targets --fail-under-lines 85` before `cargo build --release` (`FR14`, `NFR8`, `AD-6`)
**And** Stage 6 executes post-build black-box CLI integration tests against the compiled release binary (`target/release/context-sanitizer xray ...` and `--bench`) verifying end-to-end exit codes, stdout dashboard formatting, and nanosecond benchmark SLA gates across Linux, macOS, and Windows (`FR12`, `FR14`, `AD-6`).

## Epic 3: Pluggable System One ML Threat Gatekeeper & Fault-Isolated Fallback (Tier 3 E2E)

Platform security engineers can extend the pipeline via decoupled `SemanticGuard` and `TokenCompressor` traits, short-circuit structural prompt-injection and markdown exfiltration attacks in `< 100,000 ns`, or enable `--features ml` (`candle-transformers` with quantized `Llama-Prompt-Guard-2-22M` / `DeBERTa-v3-xsmall`) to detect and short-circuit semantic prompt injections across top-$K$ ($K \le 2$) surprisal-ranked 512-token overlapping windows in `~1,500,000–3,500,000 ns` median (`< 5,000,000 ns` hard `guard_timeout_ns` P99 ceiling) on CPU, with automatic zero-crash fallback to Tier 2 when model weights are unavailable, corrupt, or time out.

### Story 3.1: Pluggable Trait Decoupling (`SemanticGuard` & `TokenCompressor`) & Zero-ML Default Build Isolation

As a Systems Architect or Third-Party Crate Author,
I want `crates/context-sanitizer/src/ml/traits.rs` to define clean, thread-safe `pub trait SemanticGuard` and `pub trait TokenCompressor` interfaces while isolating all neural dependencies behind `--features ml`,
So that downstream teams can plug in custom guardrails or compressors without modifying Core, and default builds exclude Candle, PyTorch, and ONNX entirely (`0 MB` weights, `< 15 MB` binary).

**Acceptance Criteria:**

**Given** the core library `crates/context-sanitizer` compiled with default features (`cargo build` without `--features ml`)
**When** the dependency tree (`cargo tree`) and compiled binary are inspected
**Then** `pub trait SemanticGuard: Send + Sync` and `pub trait TokenCompressor: Send + Sync` are exported from `crates/context-sanitizer/src/ml/traits.rs` (`FR7`, `AD-5`)
**And** `candle-core`, `candle-transformers`, `tokenizers`, PyTorch, and ONNX crates are 100% absent from the default dependency graph (`FR7`, `NFR2`, `AD-5`).

**Given** a downstream consumer implementing custom structs for `SemanticGuard` and `TokenCompressor`
**When** those custom implementations are registered with the `context-sanitizer` pipeline builder
**Then** the pipeline invokes `evaluate_threat()` and `compress()` polymorphically through the trait boundaries, short-circuiting with `SanitizerError` when `evaluate_threat()` exceeds the configured threshold (`FR7`, `AD-5`).

### Story 3.2: Two-Stage Nanosecond Threat Gatekeeper (`< 100,000 ns` Fast-Breaker + `< 5,000,000 ns` Anti-Sponge Encoder)

As a Platform Security Engineer (Persona: Marcus, `UJ-2`),
I want a `< 100,000 ns` (`~10–25 ns/byte`) $O(N)$ SIMD fast-breaker for structural injection/exfiltration patterns paired with `--features ml` top-$K$ ($K \le 2$) surprisal-guided 512-token overlapping window inference bounded by a strict `< 5,000,000 ns` nanosecond deadline,
So that known injections are blocked in nanoseconds, split-boundary semantic jailbreaks are caught in a single forward pass, and hackers can never trigger a multi-window Neural Sponge CPU exhaustion attack.

**Acceptance Criteria:**

**Given** an incoming payload containing structural prompt-injection or exfiltration signatures (role/delimiter spoofing `<|im_start|>system`, `[INST]`, `### System:`, canonical override phrases `Ignore previous instructions`, markdown image/link exfiltration `![...](https://...)`, or base64/hex obfuscated secret smuggling)
**When** the Stage 3A-Fast lexical circuit breaker executes prior to neural inference
**Then** it detects and short-circuits the payload in `< 100,000 ns` (`~10–25 ns/byte`) in $O(N)$ linear time, returning `SanitizerError` (`InjectionBlocked`) without invoking neural tensor operations (`FR8`, `NFR1`, `AD-5`).

**Given** `--features ml` enabled with a supported System One encoder checkpoint (`crates/context-sanitizer/src/ml/{candle.rs, chunking.rs}`) and a payload exceeding 512 tokens that passes Stage 3A-Fast
**When** `chunking.rs` and `candle.rs` evaluate the payload
**Then** `chunking.rs` slices the text into 512-token windows with a configurable overlapping stride (default `128`-token overlap / `384`-token stride, preventing boundary-splitting evasion) and ranks windows by Tier 2 Shannon surprisal so at most the top-$K$ ($K \le 2$) highest-surprisal windows are evaluated (`FR8`, `AD-5`)
**And** inference executes within `~1,500,000–3,500,000 ns` median and enforces a hard nanosecond deadline (`guard_timeout_ns = 5,000,000 ns` P99 ceiling) with **Early-Exit Circuit Breaking** on the first window whose threat score crosses `--threat-threshold` (default `0.85`), immediately aborting the pipeline with `InjectionBlocked` (`FR8`, `NFR1`, `AD-5`).

### Story 3.3: Fault-Isolated Graceful Fallback to Tier 2, Offline Micro-Checkpoint Fixture & Tier 3 E2E CI Gate

As a Reliability & DevSecOps Engineer,
I want Tier 3 to gracefully fall back to Tier 2 statistical compression whenever ML model weights are missing, corrupt, unsupported, or exceed `guard_timeout_ns`—and verify the entire 3-tier cascade in CI using a deterministic offline micro-SafeTensors fixture,
So that the host application never panics or crashes in production and CI tests `--features ml` in `< 1s` without flaky network downloads.

**Acceptance Criteria:**

**Given** `--features ml` is enabled at compile time, but at runtime the configured model checkpoint file is missing, truncated/corrupt, has an invalid `tokenizer.json`, fails tensor execution, or exceeds `guard_timeout_ns`
**When** the pipeline initializes or processes a payload (`crates/context-sanitizer/src/ml/mod.rs`)
**Then** the engine emits a structured non-fatal warning log and automatically falls back to Tier 1 deterministic guard + Stage 3A-Fast (`< 100,000 ns` lexical breaker) + Tier 2 statistical compression without panicking, unwrapping, or crashing the host process (`FR9`, `NFR3`, `AD-5`).

**Given** a deterministic micro-SafeTensors + `tokenizer.json` fixture (`< 500 KB`) committed under `crates/context-sanitizer/tests/fixtures/ml/`
**When** Stage 4 (Pre-Build Unit $\to$ Integration $\to$ Coverage Gate) and Stage 6 (Post-Build CLI E2E with `--features ml`) execute in `.pre-commit-config.yaml` and `.github/workflows/ci.yml`
**Then** the Epic 3 E2E integration suite (`crates/context-sanitizer/tests/e2e_tier3_ml_gatekeeper.rs`) verifies completely offline: (1) custom `SemanticGuard`/`TokenCompressor` trait injection, (2) `< 100,000 ns` fast-breaker blocking of delimiter spoofing and markdown exfiltration, (3) $>512$-token boundary-spanning prompt injection detection, top-$K$ surprisal window bounding, and `< 5,000,000 ns` deadline enforcement, (4) Tier 1 credential redaction + Tier 3 threat blocking working together (`UJ-2`), and (5) missing/corrupt checkpoint graceful fallback to Tier 2 (`FR7`, `FR8`, `FR9`, `FR14`, `AD-5`, `AD-6`)
**And** `cargo llvm-cov --workspace --all-targets --all-features --fail-under-lines 85` passes with $\ge 85\%$ coverage before the `--features ml` release binary build (`FR14`, `NFR8`).

## Epic 4: Zero-C Python Wheel & Turnkey Agent Framework Middleware (Python E2E)

Python AI engineers (Persona: Alex, `UJ-1`) can install `context-sanitizer-py` without a C/C++ compiler toolchain and protect existing OpenAI, LangChain, CrewAI, and AutoGen agent pipelines in two lines of code with `< 50,000 ns` FFI overhead, GIL-free concurrency, and full access to `sanitize()`, opaque `VaultHandle`, and authorized tool `.rehydrate()`.

### Story 4.1: PyO3 Zero-C Core Bindings, GIL-Free Nanosecond FFI & Opaque `VaultHandle` Security

As a Python AI Systems Engineer (Persona: Alex, `UJ-1`),
I want `crates/context-sanitizer-py` to expose `sanitize()` and `VaultHandle.rehydrate()` via `pyo3` (`0.23.0`) with `< 50,000 ns` FFI overhead, full GIL release during Rust execution, and an opaque `VaultHandle` wrapper,
So that Python asyncio/multi-threaded agents achieve near-native Rust nanosecond speeds and Python crash dumps, `repr()`, or `pickle` cannot leak vaulted PII.

**Acceptance Criteria:**

**Given** `crates/context-sanitizer-py` registered in the Cargo workspace with `pyo3/extension-module` configured as a non-default Cargo feature (so `cargo test --workspace` links cleanly)
**When** Python code calls `sanitized_payload, vault = cs.sanitize(messages, rate=0.6, whitelist=["stripe_lookup"])`
**Then** the PyO3 binding releases the Python GIL (`py.allow_threads(...)`) throughout the Rust pipeline execution, adds `< 50,000 ns` FFI boundary overhead per 4KB payload, and returns the sanitized OpenAI payload alongside a session-scoped `VaultHandle` instance (`FR11`, `NFR1`, `NFR2`, `AD-2`).

**Given** a Python `VaultHandle` object holding session PII mappings
**When** a developer or malicious prompt attempts to inspect `repr(vault)`, `str(vault)`, `vault.__dict__`, or serialize it via `pickle` / `json`
**Then** `VaultHandle` exposes zero internal mapping dictionaries or salt bytes to Python (`__repr__` outputs only `<VaultHandle session_active=True>` and `pickle` raises `TypeError`), preventing accidental PII leakage into Python exception tracebacks or APM logs (`FR2`, `FR11`, `NFR5`, `AD-3`)
**And** calling `vault.rehydrate(tool_args, destination="stripe_lookup")` restores the original PII for whitelisted destinations while raising a typed `UnauthorizedRehydrationError` for unwhitelisted destinations (`FR3`, `FR11`, `AD-3`).

### Story 4.2: Turnkey Agent Framework Middleware (`LangChainSanitizer`, `CrewAISanitizer`, `AutoGenSanitizer`)

As an AI Application Developer,
I want pre-built, two-line middleware wrappers for LangChain/LangGraph, CrewAI, and AutoGen (`from context_sanitizer.integrations import LangChainSanitizer, CrewAISanitizer, AutoGenSanitizer`),
So that I can drop boundary secret redaction, PII shadow vaulting, tool argument rehydration, and context compression into existing agent pipelines without rewriting orchestration logic.

**Acceptance Criteria:**

**Given** an existing Python agent workflow using LangChain/LangGraph, CrewAI, or AutoGen
**When** a developer wraps their model or message pipeline using `LangChainSanitizer`, `CrewAISanitizer`, or `AutoGenSanitizer`
**Then** the middleware automatically intercepts outgoing message lists and `"tools"` schemas, runs the configured `context-sanitizer` cascade before LLM dispatch, short-circuits execution with a structured security exception if `InjectionBlocked` is triggered, and automatically invokes `vault.rehydrate(tool_call.args, destination=tool_call.name)` on authorized local tool calls before tool execution (`FR11`, `UJ-1`).

### Story 4.3: Zero-C Maturin Wheel Build Matrix & Clean-Virtualenv Post-Build E2E Integration Suite

As a DevSecOps & Release Engineer,
I want `.pre-commit-config.yaml` and `.github/workflows/ci.yml` to enforce Python formatting/linting (`ruff`), SAST (Semgrep/CodeQL), SCA (`pip-audit`), pre-build unit/integration coverage (`>= 85%`), zero-C `maturin` wheel builds, and post-build black-box `pytest` E2E verification across Linux, macOS, and Windows,
So that `pip install context-sanitizer-py` installs cleanly on every target OS without requiring `gcc`, `clang`, MSVC, or Rust.

**Acceptance Criteria:**

**Given** the 6-stage DevSecOps pipeline and `.pre-commit-config.yaml`
**When** Stages 1–4 execute on `crates/context-sanitizer-py`
**Then** Stage 1 enforces `ruff format --check` and `ruff check`, Stage 2 runs Semgrep/CodeQL SAST on Python wrappers, Stage 3 runs `pip-audit` SCA, and Stage 4 runs unit + integration tests with `cargo llvm-cov --workspace --fail-under-lines 85` and `pytest --cov=context_sanitizer --cov-fail-under=85` before building release wheels (`FR11`, `FR14`, `NFR8`, `AD-6`).

**Given** Stages 1–4 have passed in CI
**When** Stage 5 (`maturin build --release` targeting `manylinux_2_28`, `macos-universal2`, and `windows-x64`) and Stage 6 (Post-Build Black-Box Wheel E2E) execute
**Then** Stage 6 provisions a clean, isolated Python virtual environment with C/C++ and Rust compilers disabled/uninstalled, installs the built `.whl` via `pip install dist/*.whl`, and executes the end-to-end multi-turn agent integration test suite (`tests/e2e_python_wheel.py`) verifying `cs.sanitize()`, `vault.rehydrate()`, `LangChainSanitizer`, `CrewAISanitizer`, `AutoGenSanitizer`, and `< 50,000 ns` FFI overhead (`FR11`, `FR14`, `NFR4`, `AD-6`).

## Epic 5: Universal WebAssembly, Edge Client Sanitization & Multi-Target Release Pipeline (Wasm + Release E2E)

Frontend and edge engineers (Persona: Tara, `UJ-4`) can deploy `@context-sanitizer/wasm` (`wasm32-unknown-unknown` and `wasm32-wasip1`) inside browser extensions, Electron apps, Node.js, and Cloudflare Workers to perform client-side Tier 1 credential/PII sanitization, `< 100,000 ns` lexical injection fast-breaking, and Tier 2 statistical compression in `< 1,000,000 ns` median before payloads ever leave the client runtime, complete with the final multi-target DevSecOps release and provenance pipeline (`release.yml`).

### Story 5.1: Universal WebAssembly / WASI Bindings (`@context-sanitizer/wasm`), W3C Web Cryptography CSPRNG & Nanosecond Execution

As a Senior Frontend / Edge Engineer (Persona: Tara, `UJ-4`),
I want `wasm/` (`@context-sanitizer/wasm`) to compile the Tier 1 guard, Stage 3A-Fast lexical breaker, and Tier 2 compressor cleanly to `wasm32-unknown-unknown` and `wasm32-wasip1` via `wasm-bindgen` (`0.2.95`),
So that browser extensions, Electron apps, and Cloudflare Workers can redact secrets, vault/rehydrate PII, and compress context in `< 1,000,000 ns` median per 4KB payload before data ever crosses the network.

**Acceptance Criteria:**

**Given** the `wasm/` crate (`wasm/package.json` and `wasm/src/lib.rs`) depending unidirectionally on `crates/context-sanitizer` (with default zero-ML features)
**When** compiled via `wasm-pack build` for `wasm32-unknown-unknown` (`web`, `bundler`, `nodejs`) and `cargo build --target wasm32-wasip1`
**Then** compilation succeeds with zero C/C++ toolchain dependencies, zero native OS syscall imports, and `#![forbid(unsafe_code)]` preserved in Core (`FR13`, `NFR3`, `NFR4`, `AD-2`)
**And** the exported TypeScript/JavaScript API provides `sanitize(payloadJson, rate, whitelist)` and an opaque `WasmVaultHandle.rehydrate(toolArgs, destination)` with full `.d.ts` type definitions (`FR13`, `UJ-4`).

**Given** a `WasmVaultHandle` initialized inside a browser, Cloudflare Worker, or WASI runtime
**When** the 128-bit session salt (`[u8; 16]`) is generated using cryptographic random number generation algorithms
**Then** it sources cryptographic entropy strictly from the W3C Web Cryptography API (`crypto.getRandomValues`) CSPRNG on `wasm32-unknown-unknown` or WASI `random_get` CSPRNG on `wasm32-wasip1` (never falling back to predictable `Math.random()` or static seeds), and calling `.free()` (or JS garbage collection via `FinalizationRegistry`) immediately drops the in-memory vault from Wasm linear memory (`FR2`, `FR13`, `NFR5`, `AD-3`)
**And** client-side Tier 1 + Tier 2 execution completes in `< 1,000,000 ns` median (`< 2,500,000 ns` P99 on adversarial payloads) per 4KB payload in V8 / Wasmtime (`FR13`, `NFR1`).

### Story 5.2: Post-Build Wasm/WASI E2E Integration Suite & Multi-Target DevSecOps Release Pipeline (`release.yml`)

As a DevSecOps & Release Engineer,
I want `.github/workflows/ci.yml` and `.github/workflows/release.yml` to verify the compiled `@context-sanitizer/wasm` and `wasm32-wasip1` artifacts via post-build black-box E2E tests and gate all release artifacts (CLI binaries, zero-C Python wheels, Wasm packages, SBOMs, and SHA-256 checksums) behind the 6-stage DevSecOps pipeline,
So that every published v1.0 artifact across Crates.io, PyPI, npm, and GitHub Releases is reproducibly tested and supply-chain attested.

**Acceptance Criteria:**

**Given** the 6-stage DevSecOps pipeline in `.github/workflows/ci.yml` and `.pre-commit-config.yaml`
**When** Stage 5 (`wasm-pack build --release` and `cargo build --release --target wasm32-wasip1`) and Stage 6 (Post-Build Black-Box Wasm E2E) execute
**Then** Stage 6 loads the compiled `@context-sanitizer/wasm` package into Node.js/headless browser and `wasmtime` (for `wasm32-wasip1`), executing an end-to-end client-side workflow test (`wasm/tests/e2e_wasm_client.mjs`): (1) sanitizing an OpenAI chat payload containing synthetic cloud keys, customer PII, code blocks, and `"tools"` schemas, (2) verifying 100% root credential redaction, session-isolated `<SHADOW:*>` tokens, 40%–70% token compression, and `0%` JSON parse errors, and (3) verifying authorized vs. unauthorized `vault.rehydrate()` calls (`FR13`, `FR14`, `AD-6`).

**Given** a tagged release or workflow dispatch triggering `.github/workflows/release.yml`
**When** the release pipeline executes
**Then** it blocks artifact publishing until all 6 DevSecOps stages pass across Linux, macOS, and Windows: (1) Formatting & Linting (`cargo fmt`, `clippy`, `ruff`), (2) SAST & Secret Scanning (`#![forbid(unsafe_code)]`, panic-free lints, Semgrep/CodeQL, `gitleaks`), (3) SCA & License Audit (`cargo-audit`, `cargo-deny`, `pip-audit`), (4) Pre-Build Unit + Source Integration Tests + $\ge 85\%$ Coverage Gate (`cargo-llvm-cov` and `pytest-cov`), (5) Multi-Target Release Compilation (CLI binaries, `maturin` wheels for `manylinux_2_28`/`macos-universal2`/`windows-x64`, and `@context-sanitizer/wasm`), and (6) Post-Build Black-Box Artifact E2E Integration Tests & Benchmark SLA Gates (`FR14`, `NFR8`, `AD-6`)
**And** generates CycloneDX/SPDX SBOMs and SHA-256 checksum manifests alongside the release artifacts (`FR14`, `NFR7`, `NFR8`).

## Epic 6: Multi-Registry Publishing & Hardened Supply-Chain Release Pipeline

Enable developers worldwide to consume `context-sanitizer` through standard package managers (`cargo add context-sanitizer`, `cargo install context-sanitizer-cli`, `pip install context-sanitizer-py`) backed by automated, cryptographically signed, and TLS-hardened release pipelines protected against the October 2026 eMudhra/emSign CA compromise.

### Story 6.1: Publish `context-sanitizer` Core Library to Crates.io

As a Rust backend developer,  
I want `context-sanitizer` published to crates.io with complete documentation and package metadata,  
So that I can add it to my project using `cargo add context-sanitizer` with zero manual configuration.

**Acceptance Criteria:**

**Given** `crates/context-sanitizer/Cargo.toml`  
**When** validated for crates.io packaging  
**Then** it specifies `readme = "../../README.md"`, `categories = ["development-tools", "parsing", "web-programming"]`, `keywords = ["sanitizer", "llm", "pii", "token-compression", "prompt-security"]`, and `documentation = "https://docs.rs/context-sanitizer"` (`FR10`, `FR14`, `NFR7`).

**Given** a git release tag (`v*.*.*`) pushed to the repository  
**When** `.github/workflows/release.yml` executes Stage 5  
**Then** it runs `cargo publish -p context-sanitizer --dry-run` to verify crate integrity before executing the final `cargo publish -p context-sanitizer` using the scoped crates.io API secret (`CRATES_IO_TOKEN`) (`FR14`, `AD-6`).

**Given** the published crate on crates.io  
**When** a downstream consumer executes `cargo add context-sanitizer`  
**Then** Cargo installs version `0.1.0` and resolves all required pure-Rust dependencies (`logos`, `aho-corasick`, `serde`, `thiserror`) with zero C-toolchain requirements (`FR10`, `NFR4`).

### Story 6.2: Publish `context-sanitizer-cli` Binary to Crates.io & Multi-Arch GitHub Releases

As a DevOps engineer or security analyst,  
I want `context-sanitizer-cli` published to crates.io and distributed as precompiled multi-architecture release binaries on GitHub Releases,  
So that I can install `context-sanitizer` globally via `cargo install context-sanitizer-cli` or download standalone executables for CI/CD pipelines without compiling from source.

**Acceptance Criteria:**

**Given** `crates/context-sanitizer-cli/Cargo.toml`  
**When** prepared for crates.io publishing  
**Then** the package metadata (`description`, `readme`, `keywords`, `categories`, `repository`) is validated, and `context-sanitizer` dependency points to the published crates.io version `0.1.0` (`FR12`, `FR14`, `NFR7`).

**Given** a release workflow run  
**When** building standalone CLI executables  
**Then** it compiles optimized binaries across target matrix (`x86_64-unknown-linux-gnu`/`musl`, `aarch64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`), and generates `.tar.gz` and `.zip` archives with SHA-256 checksums uploaded to GitHub Releases (`FR12`, `FR14`, `NFR4`).

**Given** an environment with standard Cargo installed  
**When** executing `cargo install context-sanitizer-cli`  
**Then** the `context-sanitizer` binary installs into Cargo's bin directory and supports `context-sanitizer xray --help` and `context-sanitizer xray --bench` (`FR12`).

### Story 6.3: Hardened PyPI Publication for `context-sanitizer-py` with Supply-Chain TLS Mitigations

As an AI agent engineer and DevSecOps lead,  
I want `context-sanitizer-py` built and published to PyPI using strict TLS certificate verification, minimum patched tooling (`pip >= 2026.2.2`, `certifi >= 2026.10.19`, `uv >= 0.13.4`, `sigstore >= 4.5.1`), PyPI OIDC Trusted Publishing, and Sigstore provenance attestations,  
So that downstream Python users are completely protected from the October 2026 eMudhra/emSign TLS CA compromise and supply-chain MitM attacks.

**Acceptance Criteria:**

**Given** the CI release job for Python wheel distribution  
**When** configuring the build and publishing environments  
**Then** all workflow runners explicitly upgrade and pin `pip >= 2026.2.2`, `certifi >= 2026.10.19`, `uv >= 0.13.4`, and `sigstore >= 4.5.1`, and execute `update-ca-certificates` on Linux runners to purge outdated system certificate caches and unvetted eMudhra/emSign root certificates (`FR11`, `FR14`, `NFR8`).

**Given** wheel publication to PyPI  
**When** authenticating against `upload.pypi.org`  
**Then** the workflow utilizes **PyPI Trusted Publishing (OIDC)** via `pypa/gh-action-pypi-publish` with `id-token: write` permissions, eliminating static PyPI API tokens that could otherwise be intercepted across compromised TLS intermediaries (`FR11`, `FR14`, `NFR5`).

**Given** built `.whl` artifacts across Linux (`manylinux_2_28`), macOS (`macos-universal2`), and Windows (`windows-x64`)  
**When** uploaded to PyPI  
**Then** the pipeline generates **Sigstore cryptographic signatures** and SLSA provenance attestations (`attestations: true`), enabling downstream clients to cryptographically verify package integrity and publisher authenticity (`FR11`, `FR14`, `NFR8`).

**Given** a clean consumer environment  
**When** executing `pip install context-sanitizer-py`  
**Then** the wheel installs cleanly without C/C++ compilers and passes `python -c "import context_sanitizer as cs; print(cs.__version__)"` (`FR11`, `NFR4`).

### Story 6.4: PyPI Trusted Publisher Registration & Deployment Environment Verification

As a DevSecOps release engineer,  
I want `.github/workflows/release.yml` configured with an explicit GitHub `pypi` environment and matching PyPI Trusted Publisher OIDC configuration,  
So that GitHub Actions exchanges short-lived OIDC tokens directly with PyPI to authenticate wheel releases without static secrets.

**Acceptance Criteria:**

**Given** `.github/workflows/release.yml`  
**When** the `publish-pypi` job executes  
**Then** it runs within the GitHub `environment: pypi` (`url: https://pypi.org/p/context-sanitizer-py`), requests `id-token: write`, and uses `pypa/gh-action-pypi-publish@release/v1` with `attestations: true` (`FR11`, `FR14`, `NFR5`, `NFR8`).

**Given** a registered pending or active Trusted Publisher on PyPI (`reidlai/context-sanitizer`, workflow `release.yml`, environment `pypi`)  
**When** a release tag (`v*.*.*`) is pushed  
**Then** PyPI validates the OIDC identity claims (`repository: context-sanitizer`, `job_workflow_ref: .../release.yml@refs/tags/...`), mints a short-lived release token, and accepts all platform wheels without static token transmission.
