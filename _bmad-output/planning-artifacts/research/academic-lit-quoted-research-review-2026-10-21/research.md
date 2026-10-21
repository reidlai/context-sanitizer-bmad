---
title: 'academic-lit research: Quoted Research Review (README, Brainstorm, Forged Idea)'
type: 'academic-lit'
topic: 'Quoted Research Review (README, Brainstorm, Forged Idea)'
decision: 'Validate, correct, or tighten all academic citations, benchmark numbers, mathematical formulations, and Hugging Face checkpoint IDs in README.md and planning artifacts before writing SPEC.md'
source: 'native-run'
status: complete
preset: 'standard'
validation: 'normal'
claims_verified: 9
claims_unverified: 0
claims_overturned: 3
created: '2026-10-21'
updated: '2026-10-21'
---

# academic-lit research: Quoted Research Review (README, Brainstorm, Forged Idea)

**Decision this research serves:** Validate, correct, or tighten all academic citations, benchmark numbers, mathematical formulations, and Hugging Face checkpoint IDs in `README.md`, `brainstorm-intent.md`, and `forged-idea.md` before authoring `SPEC.md`.

## Executive Summary

**Verdict:** The core architectural thesis of `context-sanitizer`—combining fast encoder-based security gatekeeping (`DeBERTa-v3` / `Prompt Guard` / `ModernBERT`) with hard token compression (`LLMLingua-2` in Tier 3 and statistical/lexical pruning in Tier 2)—is strongly grounded in peer-reviewed literature (`9 verified` claims), **but 3 load-bearing claims in `README.md` are overturned by primary-source and Hugging Face Hub verification and must be corrected before `SPEC.md`:**

1. **Overturned Checkpoint ID (`HTTP 404` on Hugging Face Hub):** `README.md` (Lines 117, 177, 248, 325) uses `guard_model_id="MoritzLaurer/DeBERTa-v3-xsmall-mnli-fever-anli-ling-wanli"`. Authenticated verification against the Hugging Face Hub API confirms this repository **does not exist (`404 Not Found`)** [10]. The `-wanli` suffix only exists on the `large` checkpoint (`MoritzLaurer/DeBERTa-v3-large-mnli-fever-anli-ling-wanli`), whereas the valid `xsmall` zero-shot NLI checkpoints are **`MoritzLaurer/DeBERTa-v3-xsmall-mnli-fever-anli-ling-binary`** and **`MoritzLaurer/deberta-v3-xsmall-zeroshot-v1.1-all-33`** (built on `microsoft/deberta-v3-base` / `microsoft/deberta-v3-xsmall` [7][10]).
2. **Overturned Mathematical & Model Attribution in Tier 2 (`Selective Context` & `RECOMP`):** `README.md` Line 48 states that *Selective Context* and *RECOMP* ground the *"zero-weight Pure-Rust Tier 2 engine in information theory ($I(x) = -\log P(x)$) and query-aware BM25/submodular chunk selection... without requiring GPU weights."* In reality:
   - **Selective Context** (Li et al., EMNLP 2023) computes conditional self-information $I(x_t) = -\log_2 P(x_t \mid x_0, \dots, x_{t-1})$ using a **causal language model** (GPT-2, OPT, or LLaMA)—not a 0-weight static formula [4].
   - **RECOMP** (Xu et al., ICLR 2024) uses **trained neural compressors** (a contrastive dual-encoder extractive selector and a fine-tuned T5-Large abstractive generator) and does **not** use submodular functions or BM25 [5].
   - **Fix:** Frame Tier 2 as a **zero-weight statistical approximation inspired by *Selective Context*** that replaces neural causal-LM perplexity with **static unigram Shannon self-information / IDF**, **Okapi BM25** (Robertson & Zaragoza, 2009), and **budgeted submodular extractive selection** (Lin & Bilmes, ACL 2011) [6].
3. **Clarification on "System One Inference for Bounded Semantic Reasoning":** `README.md` Line 46 formats *"System One Inference for Bounded Semantic Reasoning"* alongside titled papers, which makes it look like a paper title. It is an original architectural concept combining Kahneman's System 1 fast pattern recognition with encoder-only transformers (`microsoft/deberta-v3-base` [7], `meta-llama/Llama-Prompt-Guard-2-22M` / `86M` [8], and `answerdotai/ModernBERT-base` [9]) and should be labeled as an architectural paradigm rather than a paper title.

---

## Dimension 1: Prompt Compression Papers & Benchmarks (`LLMLingua-2` & Survey)

- **LLMLingua-2 Paper & Formulation (`verified`, high confidence):** *LLMLingua-2: Data Distillation for Efficient and Faithful Task-Agnostic Prompt Compression* by Zhuoshi Pan et al. (Microsoft Research & Tsinghua University) was published in *Findings of the Association for Computational Linguistics: ACL 2024* (`arXiv:2403.12968`) [1]. It formulates task-agnostic prompt compression as a binary token classification problem ($p(\text{preserve})$) over a bidirectional Transformer encoder (`XLM-RoBERTa-large` and `bert-base-multilingual-cased`) trained via GPT-4 data distillation [1].
- **LLMLingua-2 Speedup & Latency Numbers (`verified`, high confidence):** All benchmark figures quoted in `README.md` Line 47 match the primary ACL 2024 paper verbatim: **3x–6x faster compression** than prior perplexity-based methods (such as original LLMLingua and Selective Context) and **1.6x–2.9x end-to-end latency acceleration** at 2x–5x compression ratios [2].
- **Prompt Compression Survey Taxonomy (`verified`, high confidence):** *Prompt Compression for Large Language Models: A Survey* by Zongqian Li, Yinhong Liu, Yixuan Su, and Nigel Collier (`arXiv:2410.12388`, Oct 2024; NAACL 2025) establishes the exact taxonomy cited in `README.md` Line 49 dividing methods into **hard prompt compression** (discrete token/lexical filtering and pruning) and **soft prompt compression** (continuous embedding vectors) [3].

## Dimension 2: Information-Theoretic & Query-Aware Pruning (`Selective Context`, `RECOMP`, Submodular Selection)

- **Selective Context Requires Causal LM Weights (`overturned` as a 0-weight claim, high confidence):** *Selective Context* (Yucheng Li, Bo Dong, Chenghua Lin, Frank Guerin, EMNLP 2023, *"Compressing Context to Enhance Inference Efficiency of Large Language Models"*, `arXiv:2310.06201`) defines token self-information as conditional surprisal $I(x_t) = -\log_2 P(x_t \mid x_0, \dots, x_{t-1})$ evaluated by a **causal language model** (GPT-2, OPT, or LLaMA) and aggregates tokens into lexical units/phrases [4]. Because `context-sanitizer` Tier 2 (`L0.5`) has **0 MB weights**, claiming it directly implements *Selective Context*'s $I(x) = -\log P(x)$ without clarifying that Tier 2 uses **static corpus unigram probability / IDF** ($I_{\text{static}}(w) = -\log P_{\text{unigram}}(w)$) rather than autoregressive conditional probability is a technical misattribution [4].
- **RECOMP Uses Neural Dual-Encoder & T5 Models, Not Submodular Selection (`overturned` attribution, high confidence):** *RECOMP: Improving Retrieval-Augmented LMs with Context Compression and Selective Augmentation* (Fangyuan Xu, Weijia Shi, Eunsol Choi, ICLR 2024, `arXiv:2310.04408`) trains two neural models: a contrastive dual-encoder sentence selector and a T5-Large abstractive compressor (with selective empty-string output when retrieved context is irrelevant) [5]. It does **not** use submodular optimization or BM25.
- **Canonical Citations for Zero-Weight Tier 2 (`verified`, high confidence):** The true mathematical foundations of Tier 2's zero-weight engine are:
  1. **Hui Lin & Jeff Bilmes (ACL-HLT 2011), *"A Class of Submodular Functions for Document Summarization"*** — which proves that monotone submodular set functions (combining relevance coverage and redundancy penalties under a token knapsack constraint) achieve a $(1 - 1/e)$ greedy approximation guarantee [6].
  2. **Stephen Robertson & Hugo Zaragoza (2009), *"The Probabilistic Relevance Framework: BM25 and Beyond"*** — the canonical citation for query-aware lexical scoring (`BM25`).

## Dimension 3: System One Security Encoders (`DeBERTaV3`, `Prompt Guard`, `ModernBERT`)

- **DeBERTaV3 Architecture & Base Checkpoints (`verified`, high confidence):** Microsoft's *DeBERTaV3* (`microsoft/deberta-v3-base`, `microsoft/deberta-v3-xsmall`, `microsoft/deberta-v3-large`), introduced by Pengcheng He, Jianfeng Gao, and Weizhu Chen at **ICLR 2023** (*"DeBERTaV3: Improving DeBERTa using ELECTRA-Style Pre-Training with Gradient-Disentangled Embedding Sharing"*, `arXiv:2111.09543`), replaces Masked Language Modeling with ELECTRA-style Replaced Token Detection (RTD) and Gradient-Disentangled Embedding Sharing (GDES), achieving 91.37% average on GLUE [7].
- **Meta Llama Prompt Guard 2 (`verified`, high confidence):** Meta's *Llama Prompt Guard 2* provides two sequence classifiers trained specifically for prompt injection and jailbreak detection with energy-based out-of-distribution (OOD) loss and adversarial tokenization hardening:
  - `meta-llama/Llama-Prompt-Guard-2-86M` (multilingual, built on `mDeBERTa-v3-base`, 86M parameters) [8].
  - `meta-llama/Llama-Prompt-Guard-2-22M` (English-focused, built on `DeBERTa-v3-xsmall`, 22M parameters, ~75% lower latency/compute overhead) [8].
- **ModernBERT (`verified`, high confidence):** *ModernBERT* (Benjamin Warner et al., `arXiv:2412.13663`, Dec 2024, *"Smarter, Better, Faster, Longer: A Modern Bidirectional Encoder for Fast, Memory Efficient, and Long Context Finetuning and Inference"*, `answerdotai/ModernBERT-base`) upgrades bidirectional encoders to an **8,192-token native context window** using Rotary Positional Embeddings (RoPE), GeGLU activations, and alternating local/global attention [9].

## Dimension 4: Hugging Face Checkpoint IDs & Rust ML Compatibility

- **`MoritzLaurer` Zero-Shot NLI Checkpoint ID Verification (`overturned` ID in `README.md`, high confidence):** Authenticated queries against the Hugging Face Hub API (`https://huggingface.co/api/models/...`) confirm that **`MoritzLaurer/DeBERTa-v3-xsmall-mnli-fever-anli-ling-wanli` returns `HTTP 404 Not Found`** [10]. Across all 19 public `DeBERTa-v3` repositories published by `MoritzLaurer`:
  - The `-wanli` non-binary checkpoint only exists on `large`: **`MoritzLaurer/DeBERTa-v3-large-mnli-fever-anli-ling-wanli`** (`HTTP 200`, 137k+ downloads) [10].
  - The valid **base** checkpoints (built on `microsoft/deberta-v3-base` [7]) are **`MoritzLaurer/DeBERTa-v3-base-mnli-fever-anli`** (`HTTP 200`, 467k+ downloads) and **`MoritzLaurer/deberta-v3-base-zeroshot-v2.0`** (`HTTP 200`, 65k+ downloads) [10].
  - The valid **xsmall** checkpoints (built on `microsoft/deberta-v3-xsmall` [7]) are **`MoritzLaurer/deberta-v3-xsmall-zeroshot-v1.1-all-33`** (`HTTP 200`, 42.9k+ downloads) and **`MoritzLaurer/DeBERTa-v3-xsmall-mnli-fever-anli-ling-binary`** (`HTTP 200`, 7.8k+ downloads) [10].
- **LLMLingua-2 Checkpoint IDs (`verified`, high confidence):** Both **`microsoft/llmlingua-2-bert-base-multilingual-cased-meetingbank`** (`HTTP 200`) and **`microsoft/llmlingua-2-xlm-roberta-large-meetingbank`** (`HTTP 200`) exist on Hugging Face Hub and are publicly accessible without gating [11].
- **Rust `candle-transformers` Architecture Support (`verified`, high confidence):** Hugging Face's pure-Rust `candle-transformers` crate natively supports:
  - `candle_transformers::models::bert` (loads `microsoft/llmlingua-2-bert-base-multilingual-cased-meetingbank`) [12].
  - `candle_transformers::models::debertav2` (loads `microsoft/deberta-v3-base`, `MoritzLaurer/DeBERTa-v3-*`, and `meta-llama/Llama-Prompt-Guard-2-*`, because DeBERTa-v3 uses `"model_type": "deberta-v2"` in Hugging Face `config.json`) [12].
  - `candle_transformers::models::modernbert` (loads `answerdotai/ModernBERT-base`) [12].

---

## Cross-Dimension Insights

1. **Context Window Asymmetry Between Security Encoders (512 tokens) and Long-Context Agent Payloads (8k+ tokens):** Both `DeBERTa-v3` (`microsoft/deberta-v3-base` [7], `MoritzLaurer/*` [10], and `Llama-Prompt-Guard-2` [8]) and `bert-base-multilingual-cased` (`LLMLingua-2` [11]) have a **512-token positional limit** (DeBERTa-v3 uses relative position buckets tuned for 512 tokens; BERT uses absolute 512 embeddings), whereas `ModernBERT` supports **8,192 tokens** natively [9]. Therefore, `SPEC.md` must specify **512-token sliding-window / chunked batching** in Tier 3 (`SemanticGuard` and `TokenCompressor`) when running `DeBERTa-v3` or `BERT` checkpoints on multi-thousand-token agent payloads!
2. **Validating the Forge Decision (`forged-idea.md`):** The decision in `forged-idea.md` to kill the "Shared-Backbone Multi-Head Model" claim from `brainstorm-intent.md` is 100% vindicated by primary sources: `Prompt Guard` / `MoritzLaurer` use `DeBERTa-v3` (`SentencePiece` `spm.model` vocabulary of 128,100 tokens) [7][8][10], while `LLMLingua-2` uses `bert-base-multilingual-cased` (`WordPiece` `vocab.txt` of 119,547 tokens) or `XLM-RoBERTa-large` (`SentencePiece` 250,002 tokens) [1][11]. They cannot share a single off-the-shelf backbone or tokenizer.

---

## Recommendations (Downstream Bindings)

1. **`README.md` (Lines 117, 177, 248, 325) & `SPEC.md` Default `guard_model_id` (`high confidence`):** Replace the non-existent `"MoritzLaurer/DeBERTa-v3-xsmall-mnli-fever-anli-ling-wanli"` (`HTTP 404`) with either:
   - **`"MoritzLaurer/deberta-v3-xsmall-zeroshot-v1.1-all-33"`** (or `"MoritzLaurer/DeBERTa-v3-xsmall-mnli-fever-anli-ling-binary"`) if defaulting to ultra-fast 22M `xsmall`, or
   - **`"MoritzLaurer/DeBERTa-v3-base-mnli-fever-anli"`** (467k downloads, built on `microsoft/deberta-v3-base` [7][10]) if defaulting to `base`.
2. **`README.md` (Lines 46–49) Research Foundation Tightening (`high confidence`):**
   - Explicitly cite **He et al. (ICLR 2023)** for *DeBERTaV3* (`microsoft/deberta-v3-base`) [7], **Meta Purple Llama (2024/2025)** for *Llama Prompt Guard 2* (`22M` / `86M`) [8], and **Warner et al. (2024)** for *ModernBERT* [9] under the System One section.
   - Reword the Tier 2 bullet (Line 48) so it states that Tier 2 is *inspired by* **Selective Context (Li et al., EMNLP 2023)** [4] and **RECOMP (Xu et al., ICLR 2024)** [5], while grounding its **0-weight pure-Rust math** in **static unigram self-information / IDF**, **Okapi BM25 (Robertson & Zaragoza, 2009)**, and **budgeted submodular extractive selection (Lin & Bilmes, ACL 2011)** [6].
3. **`SPEC.md` Tier 3 Chunking Invariant (`high confidence`):** Require Tier 3 (`SemanticGuard` and `TokenCompressor`) to chunk payloads into **$\le 512$-token windows** (stride/sentence-boundary aligned) for `DeBERTa-v3` and `BERT` checkpoints, and up to **8,192 tokens** for `ModernBERT`.

---

## Open Questions

- **DeBERTa-v3 Tokenizers in Pure Rust (`tokenizers` crate):** `microsoft/deberta-v3-base` and `MoritzLaurer/*` use a `SentencePiece` unigram tokenizer (`spm.model`) converted to `tokenizer.json`. When authoring `SPEC.md`, verify whether all target Hugging Face checkpoints ship a pre-built `tokenizer.json` file for Hugging Face's Rust `tokenizers` crate (verified: `MoritzLaurer/*` and `microsoft/llmlingua-2-*` ship `tokenizer.json`, whereas raw `microsoft/deberta-v3-base` ships `spm.model` and requires `tokenizer.json` from a fine-tuned repo).

---

## Source Appendix

| Ref | Claim / Finding Supported | Publisher | Pub Date | Accessed | Confidence |
| --- | --- | --- | --- | --- | --- |
| [1] | *LLMLingua-2: Data Distillation for Efficient and Faithful Task-Agnostic Prompt Compression* (Pan et al., ACL Findings 2024) bidirectional encoder token classification formulation | [ACL Anthology / arXiv:2403.12968](https://aclanthology.org/2024.findings-acl.57/) | 2024-08 | 2026-10-21 | high |
| [2] | LLMLingua-2 achieves 3x–6x faster compression and 1.6x–2.9x end-to-end latency speedup at 2x–5x compression ratios | [arXiv:2403.12968](https://arxiv.org/abs/2403.12968) | 2024-08 | 2026-10-21 | high |
| [3] | *Prompt Compression for Large Language Models: A Survey* (Zongqian Li et al., 2024/NAACL 2025) hard vs. soft prompt compression taxonomy | [arXiv:2410.12388](https://arxiv.org/abs/2410.12388) | 2024-10 | 2026-10-21 | high |
| [4] | *Selective Context* (Yucheng Li et al., EMNLP 2023) computes conditional self-information $I(x_t) = -\log_2 P(x_t \mid x_{<t})$ using a causal LM (GPT-2/OPT/LLaMA), not zero-weight static lookup | [ACL Anthology (EMNLP 2023)](https://aclanthology.org/2023.emnlp-main.391/) | 2023-12 | 2026-10-21 | high |
| [5] | *RECOMP: Improving Retrieval-Augmented LMs with Context Compression and Selective Augmentation* (Xu et al., ICLR 2024) uses trained dual-encoder and T5-Large compressors, not submodular selection | [OpenReview (ICLR 2024)](https://openreview.net/forum?id=mlJLVigNHp) | 2024-01 | 2026-10-21 | high |
| [6] | *A Class of Submodular Functions for Document Summarization* (Lin & Bilmes, ACL-HLT 2011) establishes budgeted submodular extractive selection with $(1 - 1/e)$ greedy guarantee | [ACL Anthology (ACL 2011)](https://aclanthology.org/P11-1052/) | 2011-06 | 2026-10-21 | high |
| [7] | *DeBERTaV3: Improving DeBERTa using ELECTRA-Style Pre-Training with Gradient-Disentangled Embedding Sharing* (He, Gao, & Chen, ICLR 2023; `microsoft/deberta-v3-base`) | [Hugging Face / OpenReview](https://huggingface.co/microsoft/deberta-v3-base) | 2023-01 | 2026-10-21 | high |
| [8] | *Meta Llama Prompt Guard 2* (`Llama-Prompt-Guard-2-86M` & `Llama-Prompt-Guard-2-22M`) DeBERTa-v3 sequence classifiers for prompt injection and jailbreak detection | [Hugging Face Hub (Meta)](https://huggingface.co/meta-llama/Llama-Prompt-Guard-2-22M) | 2025-04 | 2026-10-21 | high |
| [9] | *ModernBERT* (Warner et al., Dec 2024, `arXiv:2412.13663`) 8,192-token bidirectional encoder with RoPE and GeGLU (`answerdotai/ModernBERT-base`) | [arXiv:2412.13663](https://arxiv.org/abs/2412.13663) | 2024-12 | 2026-10-21 | high |
| [10] | Authenticated Hugging Face Hub API verification of `MoritzLaurer` `DeBERTa-v3` checkpoints (`-xsmall-...-wanli` returns 404; valid models are `deberta-v3-xsmall-zeroshot-v1.1-all-33`, `DeBERTa-v3-xsmall-mnli-fever-anli-ling-binary`, `DeBERTa-v3-base-mnli-fever-anli`, `DeBERTa-v3-large-mnli-fever-anli-ling-wanli`) | [Hugging Face Hub (MoritzLaurer)](https://huggingface.co/MoritzLaurer/deberta-v3-xsmall-zeroshot-v1.1-all-33) | 2024-02 | 2026-10-21 | high |
| [11] | Official LLMLingua-2 Hugging Face checkpoints (`microsoft/llmlingua-2-bert-base-multilingual-cased-meetingbank` & `microsoft/llmlingua-2-xlm-roberta-large-meetingbank`) verified HTTP 200 | [Hugging Face Hub (Microsoft)](https://huggingface.co/microsoft/llmlingua-2-bert-base-multilingual-cased-meetingbank) | 2024-03 | 2026-10-21 | high |
| [12] | Rust `candle-transformers` native support for `bert`, `debertav2` (covering DeBERTa-v3), and `modernbert` | [docs.rs (candle-transformers)](https://docs.rs/candle-transformers/latest/candle_transformers/models/index.html) | 2025-02 | 2026-10-21 | high |

---

## Staleness Map

Computed via `uv run scripts/recon_kit.py staleness claims.json --windows '{"version": 1, "model": 6, "benchmark": 6, "method": 12, "canon": 60}' --today 2026-10-21`:

| Claim | Class | Pub Date | Re-check Date | Stale Window Status |
| --- | --- | --- | --- | --- |
| `MoritzLaurer` DeBERTa-v3 zero-shot HF checkpoint IDs | `version` (1 mo) | 2024-02 | 2024-03-01 | Re-verified live 2026-10-21 (re-check before release) |
| `microsoft/llmlingua-2-bert-base-multilingual-cased-meetingbank` HF ID | `version` (1 mo) | 2024-03 | 2024-04-01 | Re-verified live 2026-10-21 (re-check before release) |
| `candle-transformers` support for `bert`, `debertav2`, `modernbert` | `version` (1 mo) | 2025-02 | 2025-03-01 | Re-verified live 2026-10-21 (re-check on `Cargo.toml` bump) |
| LLMLingua-2 3x–6x speedup & 1.6x–2.9x latency reduction | `benchmark` (6 mo) | 2024-08 | 2025-02-01 | Check for newer encoder compression benchmarks on Refresh |
| ModernBERT (`answerdotai/ModernBERT-base`) 8,192-token encoder | `model` (6 mo) | 2024-12 | 2025-06-01 | Check for ModernBERT token-classification fine-tunes |
| Meta Llama Prompt Guard 2 (`86M` & `22M`) classifiers | `model` (6 mo) | 2025-04 | 2025-10-01 | Check for Prompt Guard 3 or newer Purple Llama releases |
| Selective Context (Li et al., EMNLP 2023) causal LM method | `method` (12 mo) | 2023-12 | 2024-12-01 | Seminal method baseline (superseded in speed by LLMLingua-2) |
| RECOMP (Xu et al., ICLR 2024) dual-encoder/T5 method | `method` (12 mo) | 2024-01 | 2025-01-01 | Seminal RAG compression baseline |
| Lin & Bilmes (ACL 2011) submodular extractive summarization | `canon` (60 mo) | 2011-06 | 2016-06-01 | Foundational mathematical theorem ($(1-1/e)$ submodular bound) |
| DeBERTaV3 (He, Gao, & Chen, ICLR 2023) RTD + GDES | `canon` (60 mo) | 2023-01 | 2028-01-01 | Current (`< 2028-01-01`) |
| LLMLingua-2 (Pan et al., ACL Findings 2024) formulation | `canon` (60 mo) | 2024-08 | 2029-08-01 | Current (`< 2029-08-01`) |
| Prompt Compression Survey (Li et al., NAACL 2025) taxonomy | `canon` (60 mo) | 2024-10 | 2029-10-01 | Current (`< 2029-10-01`) |

*Earliest re-check date from publication window: `2016-06-01` (foundational canon); live Hugging Face Hub and `crates.io` (`version` class, 1-month window) were verified live on `2026-10-21` and should be re-checked on `2026-11-21` via `/bmad-deep-recon` Refresh.*
