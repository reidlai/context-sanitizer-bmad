# PRD Addendum: context-sanitizer

This addendum captures architectural rationale, rejected alternatives, research foundations, and technical implementation handoffs for downstream System Architecture, Solution Design, and Implementation Epics.

---

## 1. Rejected Architectural Alternatives & Rationale

| Alternative | Rationale for Rejection |
| :--- | :--- |
| **Single Shared-Backbone Multi-Head Model** | Hugging Face has no off-the-shelf model combining *Prompt Guard* / *DeBERTa-v3* `[CLS]` classification (SentencePiece 128k vocabulary) and *LLMLingua-2* token-pruning heads (WordPiece 119k / XLM-R 250k vocabulary) onto one shared encoder. Training custom multi-task weights introduces massive maintenance and infrastructure overhead. |
| **Global Predictable Shadow Tokens (`<SHADOW:EMAIL_1>`)** | Predictable sequential IDs create cross-tenant inference and prompt injection attack vectors. Session-scoped cryptographic hex hashes (e.g., `<SHADOW:EMAIL_8f3a2c>`) bound to a per-session `VaultHandle` ensure strong isolation. |
| **Rehydrating Root Credentials (`AWS_KEY`, etc.) into Tool Calls** | If root credentials can be rehydrated from shadow tokens, indirect prompt injections can force downstream tools to log or exfiltrate root keys. Root credentials must use one-way irreversible `[REDACTED_*]` masks. |
| **C-Based `tree-sitter` Grammars for Syntax Locking** | Compiling multiple C/C++ language grammars causes build fragility on Windows/macOS and breaks cross-platform precompiled wheel distribution (`maturin`). A pure-Rust lexical/JSON scanner (`logos` + `serde_json`) achieves syntax locking with zero C build dependencies. |
| **Causal LM Perplexity Pruning in Pure-Rust Tier 2** | Calculating conditional self-information $I(x_t \mid x_{<t})$ requires a full causal language model (e.g., LLaMA or GPT-2 weights), defeating the 0 MB footprint goal. Static unigram Shannon self-information / IDF and Okapi BM25 provide comparable extractive compression without neural weights. |
| **Hosted Cloud SaaS Proxy Architecture** | Routing interactive Copilot traffic through a remote SaaS gateway adds 50ms–200ms of internet round-trip latency, destroying real-time UX and introducing data-sovereignty compliance liabilities. A local-first client/proxy library has zero network latency and zero external data liability. |

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
   - *A Class of Submodular Functions for Document Summarization* (Hui Lin & Jeff Bilmes, ACL-HLT 2011). Proves monotone submodular set functions achieve $(1 - 1/e)$ approximation for knapsack-constrained budget pruning.
   - *The Probabilistic Relevance Framework: BM25 and Beyond* (Stephen Robertson & Hugo Zaragoza, 2009). Canonical formulation for query-aware Okapi BM25 scoring.
   - *Compressing Context to Enhance Inference Efficiency of Large Language Models* (Selective Context, Yucheng Li et al., EMNLP 2023, `arXiv:2310.06201`). Serves as theoretical inspiration for surprisal-based pruning, adapted in Tier 2 to static unigram Shannon self-information ($I(w) = -\log P(w)$).
4. **Correction on Overturned Checkpoints & Attributions:**
   - Verified that `MoritzLaurer/DeBERTa-v3-xsmall-mnli-fever-anli-ling-wanli` is a non-existent repo (`HTTP 404`). Validated replacements: `MoritzLaurer/deberta-v3-xsmall-zeroshot-v1.1-all-33` and `MoritzLaurer/DeBERTa-v3-base-mnli-fever-anli`.
   - Addressed 512-token positional embedding limits in `DeBERTa-v3` and `BERT`: Tier 3 enforces a 512-token sliding window with stride alignment for long agent payloads.

---

## 3. Downstream Architecture & Implementation Handoff Notes

- **Trait Interfaces:**
  - `pub trait SemanticGuard`: Must support sync/async threat evaluation, return threat probability $[0.0, 1.0]$, and allow short-circuiting before downstream calls.
  - `pub trait TokenCompressor`: Accepts structured payload or string + optional query anchor + target ratio, returning compressed content and metadata (tokens preserved, syntax locked spans).
- **Session Lifecycle & Memory Management:**
  - `VaultHandle` must be thread-safe (`Arc<RwLock<...>>`), strictly session-scoped, and deallocated immediately upon session drop.
- **Python Packaging & Maturin Pipeline:**
  - Configured via `pyproject.toml` and `maturin` targeting `manylinux_2_28`, `macos-universal2`, and `windows-x64`. Zero runtime C compiler dependencies.
- **Turnkey Integration Wrappers:**
  - LangChain: Custom `RunnableLambda` / `BaseChatModel` wrapper.
  - CrewAI: `SanitizedLLM` client intercepting agent execution scratchpads.
  - AutoGen: Registered reply transform via `TransformMessages`.
- **Wasm/WASI Compilation:**
  - `wasm32-unknown-unknown` and `wasm32-wasip1` targets for Tier 1 and Tier 2, packaged via `wasm-pack` for Node.js, Web, and Cloudflare Worker targets.
