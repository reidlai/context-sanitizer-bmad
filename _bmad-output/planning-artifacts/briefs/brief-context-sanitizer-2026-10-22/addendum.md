# Product Brief Addendum: context-sanitizer

This addendum captures technical architecture context, research foundations, and rejected alternatives for downstream artifacts (PRD, System Architecture, and Implementation Specs).

---

## 1. Rejected Architecture Alternatives & Rationale

| Alternative | Rationale for Rejection |
| :--- | :--- |
| **Single Shared-Backbone Multi-Head Model** | Hugging Face has no off-the-shelf model combining *Prompt Guard* / *DeBERTa-v3* `[CLS]` classification (SentencePiece 128k vocabulary) and *LLMLingua-2* token-pruning heads (WordPiece 119k / XLM-R 250k vocabulary) onto one shared encoder. Training or distilling custom multi-task weights introduces massive complexity and maintenance overhead. |
| **Global Predictable Shadow Tokens (`<SHADOW:EMAIL_1>`)** | Predictable sequential IDs create cross-tenant inference and prompt injection attack vectors. Session-scoped cryptographic hex hashes (e.g., `<SHADOW:EMAIL_8f3a2c>`) bound to a per-session `VaultHandle` ensure strong isolation. |
| **Rehydrating Root Credentials (`AWS_KEY`, etc.) into Tool Calls** | If root credentials can be rehydrated from shadow tokens, indirect prompt injections can force downstream tools to log or exfiltrate root keys. Root credentials must use one-way irreversible `[REDACTED_*]` masks. |
| **C-Based `tree-sitter` Grammars for Syntax Locking** | Compiling multiple C/C++ language grammars causes build fragility on Windows/macOS and breaks cross-platform precompiled wheel distribution (`maturin`). A pure-Rust lexical/JSON scanner (`logos` + `serde_json`) achieves syntax locking with zero C build dependencies. |
| **Causal LM Perplexity Pruning in Pure-Rust Tier 2** | Calculating conditional self-information $I(x_t \mid x_{<t})$ requires a full causal language model (e.g., LLaMA or GPT-2 weights), defeating the 0 MB footprint goal. Static unigram Shannon self-information / IDF and Okapi BM25 provide comparable extractive compression without neural weights. |

---

## 2. Research Foundation & Citations

1. **System One Non-Autoregressive Gatekeepers:**
   - *DeBERTaV3: Improving DeBERTa using ELECTRA-Style Pre-Training with Gradient-Disentangled Embedding Sharing* (He et al., ICLR 2023).
   - *Meta Llama Prompt Guard 2* (Meta AI, 2025).
   - *ModernBERT: A Flexible and Scalable Transformer Encoder* (Warner et al., Dec 2024).
2. **Hard Prompt Compression via Token Classification:**
   - *LLMLingua-2: Data Distillation for Efficient and Faithful Task-Agnostic Prompt Compression* (Pan et al., ACL Findings 2024, `arXiv:2403.12968`).
   - *Prompt Compression for Large Language Models: A Survey* (Zongqian Li et al., NAACL 2025, `arXiv:2410.12388`).
3. **Information-Theoretic & Submodular Pruning:**
   - *Compressing Context to Enhance Inference Efficiency of Large Language Models* (Selective Context, Li et al., EMNLP 2023, `arXiv:2310.06201`).
   - *RECOMP: Improving Retrieval-Augmented LMs with Context Compression and Selective Augmentation* (Xu et al., ICLR 2024, `arXiv:2310.04408`).
   - *Submodular Functions for Machine Learning* (Lin & Bilmes, ACL 2011).

---

## 3. Downstream PRD & Architecture Handoff Notes

- **Trait Interfaces:**
  - `pub trait SemanticGuard`: Must support async/sync execution, return threat scores and category flags, and allow short-circuiting before compression.
  - `pub trait TokenCompressor`: Must accept text + optional query anchor + target ratio, returning compressed text and metadata (tokens saved, syntax locks respected).
- **Session Lifecycle:**
  - `VaultHandle` must be thread-safe (`Arc<RwLock<...>>`) and drop cleanly upon session termination.
- **Python Packaging:**
  - Build pipeline configured using `pyproject.toml` and `maturin` targeting `manylinux`, macOS (universal2), and Windows x64.
