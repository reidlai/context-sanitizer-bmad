---
name: 'context-sanitizer'
type: architecture-spine
purpose: build-substrate
altitude: feature
paradigm: 'pipes-and-filters'
scope: 'context-sanitizer v1.0'
status: final
created: '2026-10-22'
updated: '2026-10-22'
binds: ['FR-1', 'FR-2', 'FR-3', 'FR-4', 'FR-5', 'FR-6', 'FR-7', 'FR-8', 'FR-9', 'FR-10', 'FR-11', 'FR-12', 'FR-13', 'FR-14']
sources: ['_bmad-output/planning-artifacts/prds/prd-context-sanitizer-2026-10-22/prd.md', '_bmad-output/planning-artifacts/prds/prd-context-sanitizer-2026-10-22/addendum.md']
companions: []
---

# Architecture Spine — context-sanitizer

## Design Paradigm

`context-sanitizer` is structured as a **Span-Annotated Pipes-and-Filters Cascade within a Hexagonal Envelope**. 

The core processing engine operates as an ordered sequence of zero-allocation, read-only scanners over an immutable input buffer, producing a non-overlapping span arena that is materialized in a single final allocation. External wire protocols (`POST /v1/chat/completions`, `POST /v1/chat`, `POST /v1/completions`) and platform bindings (Python PyO3, CLI, WebAssembly) interface with the engine strictly through decoupled port adapters.

```mermaid
flowchart LR
    subgraph Ingestion["Port Adapters (Hexagonal Ingestion)"]
        W1["OpenAI Chat (/v1/chat/completions, /v1/chat)"]
        W2["OpenAI Completion (/v1/completions)"]
        W3["Raw String / Payload"]
    end

    subgraph CoreEngine["Core Pipeline (Pipes & Filters over Span Arena)"]
        L0["Lexer & Syntax Locker<br/>(logos + serde_json)"] --> T1["Tier 1: Deterministic Guard<br/>(aho-corasick SIMD)"]
        T1 --> T2["Tier 2: Statistical Pruner<br/>(Shannon + Okapi BM25)"]
        T2 --> T3["Tier 3: Semantic Gatekeeper<br/>(candle-transformers)"]
        T3 --> MAT["Materializer & Fail-Open Validator<br/>(Single-Allocation Assembly)"]
    end

    subgraph VaultState["Session State"]
        VAULT["VaultHandle<br/>(Arc<RwLock<VaultInner>>)<br/>Session-Scoped PII Map"]
    end

    W1 --> L0
    W2 --> L0
    W3 --> L0
    T1 <--> VAULT
    MAT --> OUT["Sanitized Payload<br/>(Output Stream)"]
```

---

## Invariants & Rules

```mermaid
graph TD
    CLI["crates/context-sanitizer-cli<br/>(Binary: xray, bench)"] --> CORE["crates/context-sanitizer<br/>(Pure-Rust Core Library)"]
    PY["crates/context-sanitizer-py<br/>(PyO3 Python Bindings)"] --> CORE
    WASM["wasm/<br/>(WebAssembly / Browser)"] --> CORE

    CORE -.->|Forbids| CLI
    CORE -.->|Forbids| PY
    CORE -.->|Forbids| WASM
    CORE -.->|Forbids| C_DEPS["External C/C++ Compilers"]
```

### AD-1 — Span-Annotated Pipes-and-Filters Cascade

- **Binds:** `all`, `FR-1`, `FR-2`, `FR-3`, `FR-4`, `FR-5`, `FR-6`, `FR-8`
- **Prevents:** Heap reallocation churn, UTF-8 character boundary corruption, pipeline order bypasses (e.g., syntax-lock smuggling where secrets are hidden inside locked code fences/JSON keys, or pruning exposing redacted secret entropy), and ReDoS / compression-bomb algorithmic complexity attacks.
- **Rule:** Scanners execute strictly as read-only passes producing non-overlapping byte `Span` markers over an immutable payload buffer (`O(N)` linear-time DFA for Tier 1 `~20,000–100,000 ns` median / `~5–25 ns/byte`, `< 500,000 ns` P99; `O(N log N)` lazy-greedy max-heap submodular selection for Tier 2 `~300,000–400,000 ns` median / `~75–100 ns/byte`, `< 1,500,000 ns` P99, `< 500,000 ns` combined median per 4KB). `Redacted` and `Shadow` spans have absolute priority over `Locked`, `Pruned`, and `Retained` spans (preventing syntax-lock secret smuggling). Secrets are fail-closed; structural formatting is fail-open.

### AD-2 — Pure-Rust Core Isolation & Unidirectional Workspace Dependencies

- **Binds:** `all`, `FR-10`, `FR-11`, `FR-12`, `FR-13`
- **Prevents:** Circular dependencies, bloated release binaries, and cross-compilation wheel failures.
- **Rule:** The core library crate (`context-sanitizer`) must be pure Rust with `#![forbid(unsafe_code)]` in parsing cores and zero external C/C++ toolchain dependencies, compiling cleanly for native architectures (`x86_64`, `aarch64`) and WebAssembly (`wasm32-unknown-unknown`, `wasm32-wasip1`). User-facing surfaces (`cli`, `py`, `wasm`) depend unidirectionally on Core; Core depends on none of them.

### AD-3 — Session-Scoped Thread-Safe Vault with Whitelisted Rehydration

- **Binds:** `FR-2`, `FR-3`
- **Prevents:** Cross-tenant PII token collision, multi-threaded race conditions, and indirect prompt injection exfiltration to external webhooks.
- **Rule:** `VaultHandle` wraps `Arc<RwLock<VaultInner>>` utilizing a per-session 128-bit cryptographic random salt and zero disk persistence. Rehydration via `.rehydrate(args, destination)` requires an authorized destination matching a registered tool whitelist; unregistered destinations reject rehydration.

### AD-4 — Fail-Open Syntax Verification Invariant

- **Binds:** `FR-4`, `FR-6`
- **Prevents:** Downstream LLM HTTP 400 crashes caused by broken JSON escape sequences, unclosed code fences, or stripped commas.
- **Rule:** Structural JSON delimiters, markdown code fences, inline shell backticks, subshells, and environment variables receive immutable byte locks pre-pruning; post-compression output must pass `serde_json::from_str` verification or fail-open to uncompressed text with secrets remaining permanently masked.

### AD-5 — Trait Decoupling, Anti-Sponge Nanosecond Bounding & CPU Fallback for ML Gatekeeping

- **Binds:** `FR-7`, `FR-8`, `FR-9`
- **Prevents:** Mandatory multi-gigabyte PyTorch/Candle dependencies in standard builds, Neural Sponge CPU exhaustion attacks on multi-window prompts, and host application panics on unsupported CPU hardware.
- **Rule:** Core defines `pub trait SemanticGuard` and `pub trait TokenCompressor`. Structural prompt-injection and markdown exfiltration signatures are blocked in `< 100,000 ns` (`~10–25 ns/byte`) via $O(N)$ SIMD fast-breaking prior to neural execution. Candle inference (`--features ml`) evaluates at most the top-$K$ ($K \le 2$) highest-surprisal 512-token overlapping windows under a hard `guard_timeout_ns = 5,000,000 ns` P99 deadline (`~1,500,000–3,500,000 ns` median). If model weights are missing, corrupted, timeout, or fail inference, the engine logs a non-fatal warning and falls back to Tier 2 statistical compression without crashing host processes.

### AD-6 — Continuous DevSecOps Verification & Pre-Build Quality/Security Gates

- **Binds:** `all`, `FR-11`, `FR-12`, `FR-13`, `FR-14`
- **Prevents:** Unaudited transitive vulnerabilities (SCA), source code security flaws (SAST), copyleft license contamination, untested code paths, accidental real-credential commits, and unverified cross-platform wheel/Wasm builds.
- **Rule:** The repository enforces a local **`pre-commit` framework** (`.pre-commit-config.yaml`) and a strictly ordered, shift-left GitHub Actions pipeline (`.github/workflows/ci.yml` and `release.yml`) where artifact compilation is blocked until all pre-build gates pass:
  0. **Two-Speed Local `pre-commit` / `pre-push` Gates (`.pre-commit-config.yaml`):** Fast `pre-commit` hooks (`< 3s`: `cargo fmt --check`, `cargo clippy -- -D warnings`, `ruff`, `gitleaks` with `.gitleaks.toml` synthetic fixture allowlist, `cargo deny check licenses bans`) and comprehensive `pre-push` hooks (`cargo audit`, unit tests, source integration tests, and `cargo llvm-cov --fail-under-lines 85`).
  1. **Formatting & Linting:** `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `ruff format --check`, and `ruff check`.
  2. **SAST & Secret Scanning:** Compiler/Clippy AST safety lints (`#![forbid(unsafe_code)]`, `deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)` in library code), Semgrep/CodeQL static security analysis, and `gitleaks` repository secret scanning (`.gitleaks.toml`).
  3. **SCA & License Compliance:** `cargo audit`, `cargo deny check advisories bans licenses sources` (`deny.toml` enforcing MIT/Apache-2.0/BSD-3-Clause), and `pip-audit`.
  4. **Unit Tests, In-Process Source Integration Tests & Coverage Gate (Pre-Build):** Execute unit tests (`cargo test --workspace --lib --bins`) and in-process crate integration tests (`crates/*/tests/*.rs`, including offline micro-SafeTensors fixtures for `--features ml`), enforcing `cargo llvm-cov --workspace --all-targets --fail-under-lines 85` (and `pytest --cov-fail-under=85`) before any release artifact build starts. Workspace surface crates (`cli`, `py`, `wasm`) are added incrementally per epic, and `pyo3/extension-module` is gated behind a non-default feature so `cargo test --workspace` links cleanly.
  5. **Gated Artifact Build & Packaging:** Compile release binaries (`cargo build --release`), zero-C Python wheels (`maturin build --release`), and WebAssembly bundles (`wasm-pack build`).
  6. **Post-Build Black-Box Artifact E2E Integration Tests:** Run end-to-end integration tests directly against the compiled artifacts — executing the release `context-sanitizer xray` binary + `--bench` regression checks, installing the compiled `.whl` into a clean compiler-free virtualenv to run E2E agent middleware `pytest` suites, and testing the packaged `@context-sanitizer/wasm` bundle in JS/WASI runtimes.

---

## Consistency Conventions

| Concern | Convention |
| :--- | :--- |
| **Naming (Rust Types)** | `PascalCase` for types and traits (`VaultHandle`, `Span`, `SpanKind`, `SemanticGuard`, `TokenCompressor`). |
| **Naming (Rust Functions)** | `snake_case` for modules and methods (`sanitize_payload`, `rehydrate`, `evaluate_threat`). |
| **Shadow Token Format** | `<SHADOW:<TYPE>_<HEX6>>` (e.g., `<SHADOW:EMAIL_8f3a2c>`, `<SHADOW:IP_4d91b0>`, `<SHADOW:PHONE_2a1b9e>`). |
| **Redaction Mask Format** | `[REDACTED_CREDENTIAL]` for all irreversible secrets. |
| **Error Handling** | `thiserror` library enums (`SanitizerError::SyntaxLockViolation`, `SanitizerError::UnauthorizedRehydration`). No panics in library code. |
| **Wire Protocol Fields** | Exact OpenAI JSON schema parity (`messages`, `role`, `content`, `tools`, `tool_calls`). Wire paths: `POST /v1/chat/completions`, `POST /v1/chat`, `POST /v1/completions`. |
| **Licensing Policy** | MIT License for all workspace crates. All third-party dependencies must be commercially permissive (MIT, Apache-2.0, or BSD-3-Clause) and explicitly listed in `README.md`. |

---

## Stack

| Name | Version |
| :--- | :--- |
| **Rust Edition** | `2021` (MSRV 1.80+) |
| **`aho-corasick`** | `1.1.5` |
| **`logos`** | `0.16.1` |
| **`serde`** | `1.0.210` |
| **`serde_json`** | `1.0.128` |
| **`candle-core`** (`--features ml`) | `0.11.0` |
| **`candle-transformers`** (`--features ml`) | `0.11.0` |
| **`tokenizers`** (`--features ml`) | `0.20.0` |
| **`clap`** | `4.5.20` |
| **`pyo3`** | `0.23.0` |
| **`maturin`** | `1.7.4` |
| **`wasm-bindgen`** | `0.2.95` |
| **`thiserror`** | `1.0.64` |

---

## Structural Seed

### Workspace Layout

```text
context-sanitizer/
├── .github/
│   └── workflows/
│       ├── ci.yml                # Continuous DevSecOps gate (fmt, clippy, audit, deny, gitleaks, E2E matrix)
│       └── release.yml           # Cross-platform Maturin wheels, CLI binaries & Wasm release pipeline
├── .gitleaks.toml                # Secret scanning config (allowlists synthetic canary fixtures in tests/benches)
├── .pre-commit-config.yaml       # Two-speed local pre-commit/pre-push CI hooks
├── Cargo.toml                    # Root workspace configuration
├── deny.toml                     # cargo-deny policy (licenses, advisories, bans, sources)
├── crates/
│   ├── context-sanitizer/        # Core pure-Rust library
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # Public API, error types, pipeline runner
│   │       ├── arena.rs          # Span-annotated zero-copy buffer arena
│   │       ├── guard/            # Tier 1: SIMD secret patterns, entropy, VaultHandle
│   │       │   ├── mod.rs
│   │       │   ├── patterns.rs   # Multi-cloud keys, *_KEY heuristics, PII regexes
│   │       │   ├── entropy.rs    # Pure-Rust Shannon entropy (H >= 4.5)
│   │       │   └── vault.rs      # Session-scoped VaultHandle & destination whitelisting
│   │       ├── compress/         # Tier 2: Lexical syntax locking, Shannon, BM25, schema minifier
│   │       │   ├── mod.rs
│   │       │   ├── lexer.rs      # logos syntax token locking (JSON, code fences, shell backticks)
│   │       │   ├── shannon.rs    # Static unigram self-information I(w) = -log P(w)
│   │       │   ├── bm25.rs       # Okapi BM25 tail-query scoring
│   │       │   ├── submodular.rs # Budgeted knapsack chunk selection
│   │       │   └── schema.rs     # JSON tool schema minifier
│   │       ├── ml/               # Tier 3: Trait definitions & Candle inference (--features ml)
│   │       │   ├── mod.rs
│   │       │   ├── traits.rs     # pub trait SemanticGuard & TokenCompressor
│   │       │   ├── candle.rs     # Candle encoder runner (DeBERTa / Prompt Guard 2)
│   │       │   └── chunking.rs   # 512-token sliding window with stride alignment
│   │       └── wire/             # Wire serialization
│   │           ├── mod.rs
│   │           ├── chat.rs       # /v1/chat/completions and /v1/chat models
│   │           └── completion.rs # /v1/completions prompt models
│   ├── context-sanitizer-cli/    # Standalone binary crate (CLI & benchmarks)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs           # Entry point
│   │       ├── xray.rs           # Terminal visualization (tables, colors, savings math)
│   │       └── bench.rs          # Deterministic benchmarks (MeetingBank, GSM8K)
│   └── context-sanitizer-py/     # PyO3 Python bindings crate
│       ├── Cargo.toml
│       ├── pyproject.toml
│       ├── src/
│       │   └── lib.rs            # PyO3 exports: sanitize, rehydrate, VaultHandle
│       └── python/
│           └── context_sanitizer/
│               ├── __init__.py
│               └── integrations/ # Turnkey middleware wrappers
│                   ├── langchain.py
│                   ├── crewai.py
│                   └── autogen.py
├── wasm/                         # WebAssembly package (@context-sanitizer/wasm)
│   ├── package.json
│   └── src/
│       └── lib.rs                # wasm-bindgen exports for browser/edge runtimes
└── benches/                      # Criterion microbenchmarks
    └── pipeline_throughput.rs
```

### Core Rust Entity Signatures

```rust
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub kind: SpanKind,
}

pub enum SpanKind {
    Locked,
    Redacted(&'static str),
    Shadow(String),
    Pruned,
    Retained,
}

pub struct VaultHandle {
    inner: Arc<RwLock<VaultInner>>,
}

struct VaultInner {
    forward: HashMap<String, String>,
    reverse: HashMap<String, String>,
    whitelist: HashSet<String>,
    salt: [u8; 16],
}

#[async_trait]
pub trait SemanticGuard: Send + Sync {
    async fn evaluate_threat(&self, text: &str) -> Result<f32, SanitizerError>;
}

pub trait TokenCompressor: Send + Sync {
    fn compress(&self, payload: &str, target_rate: f32) -> Result<String, SanitizerError>;
}
```

---

## Capability → Architecture Map

| Capability / Area | Lives in | Governed by |
| :--- | :--- | :--- |
| **FR-1: Credential Interception** | `crates/context-sanitizer/src/guard/patterns.rs`, `entropy.rs` | AD-1 (Redaction priority), AD-2 (Pure Rust) |
| **FR-2: PII Shadow Vaulting** | `crates/context-sanitizer/src/guard/vault.rs` | AD-3 (Session-scoped isolation) |
| **FR-3: Whitelisted Rehydration** | `crates/context-sanitizer/src/guard/vault.rs` | AD-3 (Destination whitelist) |
| **FR-4: Syntax Locking & Fail-Open** | `crates/context-sanitizer/src/compress/lexer.rs` | AD-1 (Span arena), AD-4 (Fail-open) |
| **FR-5: Statistical Pruning** | `crates/context-sanitizer/src/compress/shannon.rs`, `bm25.rs` | AD-1 (Read-only span markings) |
| **FR-6: JSON Schema Minification** | `crates/context-sanitizer/src/compress/schema.rs` | AD-4 (JSON schema preservation) |
| **FR-7: Decoupled ML Traits** | `crates/context-sanitizer/src/ml/traits.rs` | AD-5 (Pluggable traits) |
| **FR-8: Two-Stage Nanosecond Threat Gatekeeper** | `crates/context-sanitizer/src/ml/candle.rs`, `chunking.rs` | AD-5 (`< 100,000 ns` fast-breaker + top-$K \le 2$ surprisal window under `< 5,000,000 ns` `guard_timeout_ns`) |
| **FR-9: Fault-Tolerant Fallback** | `crates/context-sanitizer/src/ml/mod.rs` | AD-5 (Zero-crash fallback to Tier 2) |
| **FR-10: OpenAI Wire Ingestion** | `crates/context-sanitizer/src/wire/` | Consistency Conventions (OpenAI Parity) |
| **FR-11: Zero-C Python Wheel** | `crates/context-sanitizer-py/` | AD-2 (Maturin precompiled wheels) |
| **FR-12: Context X-Ray CLI** | `crates/context-sanitizer-cli/` | AD-2 (Unidirectional dependency) |
| **FR-13: Universal Wasm Package** | `wasm/` | AD-2 (Pure-Rust Wasm compilation) |
| **FR-14: Automated DevSecOps CI/CD** | `.github/workflows/`, `.pre-commit-config.yaml`, `deny.toml` | AD-6 (Local pre-commit + Continuous DevSecOps gates) |

---

## Deferred

| Item | Reason for Deferral |
| :--- | :--- |
| **Streaming Token Chunk Compression** | Requires maintaining stateful lexical window buffers across chunk boundaries. Can be layered in v2 without breaking the batch API. |
| **Cross-Restart Persistent Vaults** | Ephemeral in-memory vaults eliminate disk encryption overhead and cross-tenant leakage. Persistent vaults belong in enterprise storage layers. |
| **Distributed Multi-Node Vault Sync** | A local proxy architecture operates per-instance with zero network hops. Multi-node cluster replication is deferred to centralized gateway tiers. |
