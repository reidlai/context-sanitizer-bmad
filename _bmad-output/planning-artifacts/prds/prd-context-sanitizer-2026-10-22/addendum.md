# PRD Addendum: context-sanitizer

This addendum captures architectural rationale, rejected alternatives, research foundations, and technical implementation handoffs for downstream System Architecture, Solution Design, and Implementation Epics.

---

## 1. Rejected Architectural Alternatives & Rationale

| Alternative | Rationale for Rejection |
| :--- | :--- |
| **Single Shared-Backbone Multi-Head Model** | Hugging Face has no off-the-shelf model combining *Prompt Guard* / *DeBERTa-v3* `[CLS]` classification (SentencePiece 128k vocabulary) and *LLMLingua-2* token-pruning heads (WordPiece 119k / XLM-R 250k vocabulary) onto one shared encoder. Training custom multi-task weights introduces massive maintenance and infrastructure overhead. |
| **Global Predictable Shadow Tokens (`<SHADOW:EMAIL_1>`)** | Predictable sequential IDs create cross-tenant inference and prompt injection attack vectors. Session-scoped cryptographic hex hashes (e.g., `<SHADOW:EMAIL_8f3a2c>`) bound to a per-session `VaultHandle` ensure strong isolation. |
| **Rehydrating Root Credentials (`AWS_KEY`, etc.) into Tool Calls** | If root credentials can be rehydrated from shadow tokens, indirect prompt injections can force downstream tools to log or exfiltrate root keys. Root credentials must use one-way irreversible `[REDACTED_CREDENTIAL]` masks. |
| **C-Based `tree-sitter` Grammars for Syntax Locking** | Compiling multiple C/C++ language grammars causes build fragility on Windows/macOS and breaks cross-platform precompiled wheel distribution (`maturin`). A pure-Rust lexical/JSON scanner (`logos` + `serde_json`) achieves syntax locking with zero C build dependencies. |
| **Causal LM Perplexity Pruning in Pure-Rust Tier 2** | Calculating conditional self-information $I(x_t \mid x_{<t})$ requires a full causal language model (e.g., LLaMA or GPT-2 weights), defeating the 0 MB footprint goal. Static unigram Shannon self-information / IDF and Okapi BM25 provide comparable extractive compression without neural weights. |
| **Hosted Cloud SaaS Proxy Architecture** | Routing interactive Copilot traffic through a remote SaaS gateway adds 50,000,000–200,000,000 ns (50ms–200ms) of internet round-trip latency, destroying real-time UX and introducing data-sovereignty compliance liabilities. A local-first client/proxy library has zero network latency and zero external data liability. |

---

## 2. Peer-Reviewed Research Foundations & Corrections

1. **System One Non-Autoregressive Gatekeepers:**
   - *DeBERTaV3: Improving DeBERTa using ELECTRA-Style Pre-Training with Gradient-Disentangled Embedding Sharing* (Pengcheng He, Jianfeng Gao, Weizhu Chen, ICLR 2023, `arXiv:2111.09543`).
   - *Meta Llama Prompt Guard 2* (Meta AI, 2025; checkpoints `meta-llama/Llama-Prompt-Guard-2-22M` and `86M`).
   - *ModernBERT: A Flexible and Scalable Transformer Encoder* (Benjamin Warner et al., Dec 2024, `arXiv:2412.13663`, `answerdotai/ModernBERT-base`).
2. **Hard Prompt Compression via Token Classification:**
   - *LLMLingua-2: Data Distillation for Efficient and Faithful Task-Agnostic Prompt Compression* (Zhuoshi Pan et al., Microsoft Research & Tsinghua University, ACL Findings 2024, `arXiv:2403.12968`).
   - *Prompt Compression for Large Language Models: A Survey* (Zongqian Li, Yinhong Liu, Yixuan Su, Nigel Collier, NAACL 2025, `arXiv:2410.12388`).
3. **Information-Theoretic & Submodular Pruning:**
   - *A Class of Submodular Functions for Document Summarization* (Hui Lin & Jeff Bilmes, ACL-HLT 2011). Proves monotone submodular set functions achieve $(1 - 1/e)$ approximation for knapsack-constrained budget pruning via $O(N \log N)$ lazy-greedy max-heap selection.
   - *The Probabilistic Relevance Framework: BM25 and Beyond* (Stephen Robertson & Hugo Zaragoza, 2009). Canonical formulation for query-aware Okapi BM25 scoring.
   - *Compressing Context to Enhance Inference Efficiency of Large Language Models* (Selective Context, Yucheng Li et al., EMNLP 2023, `arXiv:2310.06201`). Serves as theoretical inspiration for surprisal-based pruning, adapted in Tier 2 to static unigram Shannon self-information ($I(w) = -\log P(w)$).
4. **Correction on Overturned Checkpoints, Anti-Sponge Bounding & Attributions:**
   - Verified that `MoritzLaurer/DeBERTa-v3-xsmall-mnli-fever-anli-ling-wanli` is a non-existent repo (`HTTP 404`). Validated replacements: `MoritzLaurer/deberta-v3-xsmall-zeroshot-v1.1-all-33` and `MoritzLaurer/DeBERTa-v3-base-mnli-fever-anli`.
   - Addressed 512-token positional embedding limits and Neural Sponge CPU exhaustion in `DeBERTa-v3` and `Prompt-Guard-2`: Tier 3 enforces a `< 100,000 ns` (`~10–25 ns/byte`) $O(N)$ SIMD fast-breaker followed by a 512-token sliding window with stride alignment that evaluates at most the top-$K$ ($K \le 2$) highest-surprisal windows under a hard `guard_timeout_ns = 5,000,000 ns` P99 ceiling (`~1,500,000–3,500,000 ns` median).

---

## 3. Downstream Architecture & Implementation Handoff Notes

- **Trait Interfaces (`SanitizerError` Unified Error Type):**
  - `pub trait SemanticGuard`: `async fn evaluate_threat(&self, text: &str) -> Result<f32, SanitizerError>;` — returns threat probability $[0.0, 1.0]$ and short-circuits before downstream calls.
  - `pub trait TokenCompressor`: `fn compress(&self, payload: &str, target_rate: f32) -> Result<String, SanitizerError>;` — returns compressed content while respecting immutable syntax-locked byte spans.
- **Nanosecond Latency & Anti-DoS Complexity Bounds (`NFR-1`):**
  - Tier 1 (`L0`): $O(N)$ linear-time DFA (`aho-corasick`), `~20,000–100,000 ns` median (`~5–25 ns/byte`), `< 500,000 ns` P99 per 4KB.
  - Tier 2 (`L0.5`): $O(N \log N)$ lazy-greedy max-heap selection + `logos` DFA, `~300,000–400,000 ns` median (`~75–100 ns/byte`), `< 1,500,000 ns` P99 per 4KB (`< 500,000 ns` combined L0+L0.5 median).
  - Tier 3 (`L1`): `< 100,000 ns` (`~10–25 ns/byte`) Stage 3A-Fast lexical breaker + `~1,500,000–3,500,000 ns` median (`< 5,000,000 ns` hard `guard_timeout_ns` P99 ceiling across top-$K \le 2$ surprisal windows).
- **Session Lifecycle & Memory Management:**
  - `VaultHandle` must be thread-safe (`Arc<RwLock<VaultInner>>`), strictly session-scoped with a 128-bit CSPRNG salt, and deallocated immediately upon session drop.
- **Python Packaging & Maturin Pipeline:**
  - Configured via `pyproject.toml` and `maturin` targeting `manylinux_2_28`, `macos-universal2`, and `windows-x64` (`pip install context-sanitizer-py`). Zero runtime C compiler dependencies (`< 50,000 ns` PyO3 FFI boundary overhead per 4KB).
- **Turnkey Integration Wrappers:**
  - LangChain: `LangChainSanitizer` (`RunnableLambda` / `BaseChatModel` wrapper).
  - CrewAI: `CrewAISanitizer` (`SanitizedLLM` client intercepting agent execution scratchpads).
  - AutoGen: `AutoGenSanitizer` (registered reply transform via `TransformMessages`).
- **Wasm/WASI Compilation:**
  - `wasm32-unknown-unknown` and `wasm32-wasip1` targets for Tier 1 and Tier 2, packaged via `wasm-pack` as `@context-sanitizer/wasm` (`< 1,000,000 ns` median / `< 2,500,000 ns` P99 per 4KB, using W3C Web Cryptography API `crypto.getRandomValues` for CSPRNG session salts).
- **DevSecOps CI/CD & Two-Speed Local `pre-commit` Pipeline (`FR-14` / `AD-6`):**
  - Stage 0: `.pre-commit-config.yaml` (`pre-commit` `< 3s` fast checks + `pre-push` unit/integration/coverage gates) and `.gitleaks.toml` synthetic fixture allowlist.
  - Stages 1–4 (Pre-Build Gates): Formatting & Linting (`cargo fmt`, `clippy -D warnings`, `ruff`), SAST (`#![forbid(unsafe_code)]`, panic-free lints, Semgrep/CodeQL, `gitleaks`), SCA (`cargo audit`, `cargo deny`, `pip-audit`), and Unit $\to$ In-Process Integration $\to$ $\ge 85\%$ Coverage Gate (`cargo llvm-cov` and `pytest-cov`).
  - Stages 5–6 (Post-Build Gates): Gated artifact build (`cargo build --release`, `maturin build --release`, `wasm-pack build`) followed by post-build black-box E2E integration and benchmark verification across CLI, compiler-free Python `.whl` virtualenvs, and JS/WASI runtimes.
