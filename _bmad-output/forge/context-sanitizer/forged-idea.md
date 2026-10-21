# Forged Idea: `context-sanitizer` (HARDENED)

## Locked Decisions
- **Tier 3 (`--features ml`) Sequential Off-the-Shelf Pipeline:** Run existing Hugging Face checkpoints sequentially—`SemanticGuard` runs first and short-circuits on threats; `TokenCompressor` (*LLMLingua-2*) runs second only on allowed payloads.
- **Trait-Based Configurable Model Architecture:** Expose `pub trait SemanticGuard` and `pub trait TokenCompressor` with configurable `guard_model_id` and `compression_model_id` (defaulting to `DeBERTa-v3` Zero-Shot NLI + L0 custom terms for `blocked_categories`, and `LLMLingua-2` BERT for token pruning) so users can swap in *ModernBERT*, *Prompt Guard*, or custom ONNX/Candle models without code changes.
- **Session-Scoped Shadow Vault + Split Secret Policy:**
  - Generate session-scoped cryptographic tokens (`<SHADOW:EMAIL_8f3a2c>`) tied to a per-session `VaultHandle` with an explicit `.rehydrate()` method on the Rust and Python APIs.
  - **Operational PII** (`EMAIL`, `IP`, `PHONE`, `USER_ID`) uses the Two-Way Shadow Vault for local tool-call rehydration.
  - **Root Credentials** (`AWS_KEY`, `GITHUB_TOKEN`, `PRIVATE_KEY`) default to One-Way Redaction (`[REDACTED_*]`) so indirect prompt injection cannot exfiltrate root keys via tool arguments.
- **100% Pure-Rust Lexical/JSON Scanner & 3-Crate Workspace:**
  - Use a pure-Rust lexical/JSON scanner (`logos` + `serde_json`) to lock code/JSON syntax tokens and detect code injection patterns with zero C/C++ build dependencies.
  - Structure repository as a 3-crate Cargo workspace: `context-sanitizer` (core library), `context-sanitizer-cli` (`context-sanitizer xray` binary), and `context-sanitizer-py` (`PyO3` bindings).
- **Tier 2 (`L0.5`) Tail-Query Auto-Anchoring:** When a single prompt string is passed without an explicit `query`, automatically lock the final user question/instruction as the query anchor for BM25/self-information chunk scoring alongside static English token-density filtering.

## Rejected Options & Why
- **Killed: Single Shared-Backbone Multi-Head Checkpoint:** No off-the-shelf Hugging Face model combines *Prompt Guard* `[CLS]` security heads and *LLMLingua-2* token-pruning heads on one shared vocabulary/encoder, and training/distilling custom weights is out of scope.
- **Killed: Global Predictable Shadow Vault IDs (`<SHADOW:EMAIL_1>`) & Rehydrating Root Keys:** Predictable global IDs allow cross-tenant prompt injection leaks, and rehydrating root credentials into tool calls allows exfiltration via indirect prompt injection.
- **Killed: C-Based `tree-sitter` Grammar Dependencies:** Compiling multiple C/C++ language grammars inflates compile times and breaks cross-platform `maturin` wheel builds; pure-Rust lexical scanning achieves structural token locking cleanly.
