# Epic 1 Context: Deterministic Boundary Guard, Split-Secret Vault, OpenAI Wire Pipeline & DevSecOps CI Foundation (Tier 1 E2E)

<!-- Generated from planning artifacts. Regenerate with compile-epic-context if planning docs change. -->

## Goal

Enable AI systems engineers to pass raw strings or OpenAI-compatible payloads (`/v1/chat/completions`, `/v1/chat`, `/v1/completions`) through a pure-Rust, span-annotated pipeline that permanently redacts root credentials as `[REDACTED_CREDENTIAL]`, vaults operational PII into session-isolated `<SHADOW:<TYPE>_<HEX6>>` tokens within `~20,000–100,000 ns` median (`< 500,000 ns` P99 per 4KB), and safely rehydrates authorized local tool call arguments via `VaultHandle::rehydrate()`, backed from the first commit by a two-speed local `pre-commit` framework and a 6-stage DevSecOps CI pipeline.

## Stories

- Story 1.1: Workspace Foundation, Span Arena & Two-Speed DevSecOps CI/Pre-Commit Spine
- Story 1.2: Tier 1 Comprehensive Credential Interception, Irreversible Redaction & Anti-ReDoS Guarantee
- Story 1.3: Operational PII Detection, Cryptographic Session Shadow Vault & Whitelisted Rehydration
- Story 1.4: OpenAI-Compatible Wire Ingestion, Tier 1 E2E Integration Suite & CI Gate Verification

## Requirements & Constraints

- **Comprehensive Credential Interception & One-Way Redaction:** Detect and irreversibly mask root secrets with `[REDACTED_CREDENTIAL]` without storing raw values in any vault, struct, or log:
  - Known vendor signatures: AWS (`AKIA...`), GCP (`AIza...` and service account private keys), Azure storage/tenant secrets, Kubernetes bearer tokens and TLS certs, Okta (`00...`), Salesforce, GitHub (`ghp_...`), GitLab, Slack (`xoxb-`, `xoxp-`), and PGP/SSH private key blocks (`-----BEGIN ... PRIVATE KEY-----`).
  - Case-insensitive variable assignments and JSON/YAML key-value pairs matching `*_KEY`, `*_SECRET`, `*_TOKEN`, `*_AUTH`, `*_PASS`, `*_PASSWORD`, `*_CREDENTIAL`, `*_APIKEY`, or `*_PRIVATE_KEY` where the assigned value is $\ge 16$ characters.
  - Standalone unanchored tokens with length $\ge 20$ and Shannon entropy $H \ge 4.5$.
  - Zero false positives on benign code identifiers or short values (e.g., `const KEY_COUNT = 5`, `token_limit = 4096`).
- **Operational PII Shadow Vaulting:** Detect email addresses, IPv4/IPv6 addresses, and telephone numbers, replacing each unique entity within a session with `<SHADOW:<TYPE>_<HEX6>>` (e.g., `<SHADOW:EMAIL_8f3a2c>`, `<SHADOW:IP_4d91b0>`, `<SHADOW:PHONE_2a1b9e>`). Identical PII strings within the same session must deterministically map to the same token, while concurrent sessions processing identical PII must produce distinct tokens using a 128-bit cryptographic session salt.
- **Whitelisted Local Tool Rehydration:** `VaultHandle::rehydrate(args, destination_tool)` must replace shadow tokens with raw PII only when `destination_tool` matches an entry in the session's authorized tool whitelist, returning `Err(SanitizerError::UnauthorizedRehydration)` and zero raw PII for unregistered or external destinations.
- **OpenAI Wire Parity:** Ingest, sanitize, and re-serialize structured chat payloads (`POST /v1/chat/completions` and gateway alias `POST /v1/chat` preserving `messages`, `role`, `content`, `tools`, `tool_calls`) and legacy completion payloads (`POST /v1/completions` `prompt` strings), preserving 100% schema parity and unknown pass-through fields.
- **Performance, Anti-ReDoS & Concurrency SLAs:** Tier 1 scanning must execute in strict $O(N)$ linear time (`~20,000–100,000 ns` median / `~5–25 ns/byte`, `< 500,000 ns` P99 ceiling per 4KB payload even on adversarial near-match ReDoS inputs) with zero backtracking regexes and zero persistent heap allocations outside active sessions. `VaultHandle` must support 10,000+ concurrent requests/sec without lock contention or data races, deallocating all mappings immediately when dropped.
- **Safety, Licensing & Portability:** Enforce `#![forbid(unsafe_code)]` and `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]` in library code with zero C/C++ compiler dependencies. Use MIT licensing across the workspace and restrict third-party dependencies to commercially permissive licenses (`MIT`, `Apache-2.0`, `BSD-3-Clause`) documented in `README.md`.
- **Two-Speed Local Hooks & 6-Stage CI Gate:**
  - Local `.pre-commit-config.yaml`: Fast `< 3s` `pre-commit` hooks (`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `gitleaks` with `.gitleaks.toml` allowlisting `**/tests/fixtures/**` and `benches/**`, `cargo deny check licenses bans`) and `pre-push` hooks (`cargo audit`, unit tests, integration tests, `cargo llvm-cov --workspace --all-targets --fail-under-lines 85`).
  - GitHub Actions `.github/workflows/ci.yml` across Linux, macOS, and Windows: (1) Formatting & Linting, (2) SAST & Secret Scanning (`clippy` safety lints, Semgrep/CodeQL, `gitleaks`), (3) SCA (`cargo audit`, `cargo deny check advisories bans licenses sources` via `deny.toml`), (4) Pre-Build Unit Tests + In-Process Integration Tests + $\ge 85\%$ line coverage gate (`cargo llvm-cov`), (5) Gated `cargo build --release`, and (6) Post-Build verification.

## Technical Decisions

- **Incremental Workspace Activation:** Initialize the root `Cargo.toml` on Rust Edition 2021 (MSRV 1.80+) with only `crates/context-sanitizer` (Core) active in Epic 1 so workspace-wide coverage gates ($\ge 85\%$) never fail on unbuilt surface crates (`cli`, `py`, `wasm` are registered in later epics).
- **Pinned Core Dependencies:** Use `aho-corasick 1.1.5` (SIMD DFA pattern matching), `serde 1.0.210`, `serde_json 1.0.128`, and `thiserror 1.0.64`.
- **Span-Annotated Zero-Copy Arena (`crates/context-sanitizer/src/arena.rs`):**
  - Core types: `Span { pub start: usize, pub end: usize, pub kind: SpanKind }` and `SpanKind { Locked, Redacted(&'static str), Shadow(String), Pruned, Retained }`.
  - All scanners run as read-only passes over an immutable input buffer; the materializer resolves overlaps and assembles the final sanitized string in a single allocation without corrupting multi-byte UTF-8 boundaries.
  - Precedence invariant: `Redacted` and `Shadow` spans hold absolute priority over `Locked`, `Pruned`, and `Retained` spans.
- **Cryptographic Session Vault (`crates/context-sanitizer/src/guard/vault.rs`):**
  - `VaultHandle` wraps `Arc<RwLock<VaultInner>>` containing `forward: HashMap<String, String>`, `reverse: HashMap<String, String>`, `whitelist: HashSet<String>`, and per-session `salt: [u8; 16]`.
- **Module Layout & Error Conventions (`crates/context-sanitizer/src/`):**
  - `lib.rs` (public API, pipeline runner, `SanitizerError` enum via `thiserror` including `SanitizerError::UnauthorizedRehydration` and `SanitizerError::SyntaxLockViolation`), `arena.rs`, `guard/{mod.rs, patterns.rs, entropy.rs, vault.rs}`, and `wire/{mod.rs, chat.rs, completion.rs}`.
  - Follow `PascalCase` for types/traits and `snake_case` for functions/modules (`sanitize_payload`, `rehydrate`).

## Cross-Story Dependencies

- **Story 1.1 $\to$ Stories 1.2, 1.3, 1.4:** Story 1.1 establishes the Cargo workspace, `SanitizerError`, `Span`/`SpanKind` arena materializer, and CI/pre-commit configuration required by all subsequent stories.
- **Stories 1.2 & 1.3 $\to$ Story 1.4:** Story 1.2 (`guard/patterns.rs`, `guard/entropy.rs`) and Story 1.3 (`guard/vault.rs`) provide the Tier 1 credential redaction and PII shadow vaulting passes orchestrated by Story 1.4's OpenAI wire adapters (`wire/chat.rs`, `wire/completion.rs`) and verified in `crates/context-sanitizer/tests/e2e_tier1_pipeline.rs`.
- **Epic 1 $\to$ Epics 2–5:** The `Span` arena precedence rules, `VaultHandle` API, OpenAI wire models, and 6-stage DevSecOps CI spine built in Epic 1 serve as the foundational substrate for Tier 2 statistical compression and CLI (Epic 2), Tier 3 ML gatekeeping (Epic 3), Python bindings (Epic 4), and Wasm packaging (Epic 5).
