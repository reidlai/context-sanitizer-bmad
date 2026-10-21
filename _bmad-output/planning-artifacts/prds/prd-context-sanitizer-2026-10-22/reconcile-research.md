# Input Reconciliation: Academic Lit Research

- **Source Input:** `_bmad-output/planning-artifacts/research/academic-lit-quoted-research-review-2026-10-21/research.md`
- **Reconciliation Target:** `prd.md` + `addendum.md`
- **Status:** Complete / Aligned

## Gaps & Nuance Analysis

1. **Overturned Checkpoints (Corrected):** Validated checkpoints (`meta-llama/Llama-Prompt-Guard-2-22M`, `MoritzLaurer/deberta-v3-xsmall-zeroshot-v1.1-all-33`, `microsoft/llmlingua-2-bert-base-multilingual-cased-meetingbank`) used in place of the overturned 404 ID in PRD FR-8 and Addendum §2.
2. **512-Token Positional Limit (Codified):** Sliding-window chunking requirement for DeBERTa/BERT encoder models explicitly codified in FR-8 and Addendum §2.
3. **Information Theory Attribution (Precision):** Clarified distinction between causal-LM conditional surprisal (Selective Context) and zero-weight static unigram Shannon self-information ($I(w) = -\log P(w)$) + Okapi BM25 + submodular optimization (Lin & Bilmes 2011) in FR-5 and Addendum §2.
