---
title: "Sprint Change Proposal — Cross-Artifact Synchronization & Course Correction"
project: "context-sanitizer"
date: "2026-10-22"
status: "approved-and-applied"
mode: "batch"
---

# Sprint Change Proposal: Cross-Artifact Synchronization (`context-sanitizer`)

## 1. Issue Summary & Change Trigger

Following the completion of the Epics & Stories breakdown ([epics.md](file:///c:/Users/reidl/GitLocal/context-sanitizer/_bmad-output/planning-artifacts/epics.md)), three major architectural and quality refinements were established with the user (`Reid`):
1. **End-to-End Integration-Testable Epic Grouping:** Organizing all 16 stories into 5 cohesive epics where each epic culminates in an end-to-end integration test.
2. **Two-Speed Local `pre-commit` Framework + 6-Stage Shift-Left & Post-Build DevSecOps Pipeline (`FR-14` / `AD-6`):** Enforcing `.pre-commit-config.yaml` (`pre-commit` `< 3s` fast checks + `pre-push` unit/integration/coverage gates) and a 6-stage GitHub Actions pipeline where formatting, linting, SAST, SCA, unit tests, in-process integration tests, and $\ge 85\%$ line coverage gates (`cargo-llvm-cov` and `pytest-cov`) must pass **before** any artifact build (`cargo build --release`, `maturin`, `wasm-pack`), followed by post-build black-box E2E integration and benchmark verification.
3. **Nanosecond (`ns`) Latency SLAs & Anti-DoS Complexity Bounds (`NFR-1`, `SM-2`, `FR-8`, `FR-12`):** Replacing coarse millisecond (`ms`) thresholds with nanosecond (`ns`) and `ns/byte` SLAs (`std::time::Instant::elapsed().as_nanos()`) paired with strict algorithmic complexity bounds:
   - **Tier 1 (`L0`):** $O(N)$ linear-time SIMD DFA (`aho-corasick`), `~20,000–100,000 ns` median (`~5–25 ns/byte`), `< 500,000 ns` P99 ceiling per 4KB (Anti-ReDoS).
   - **Tier 2 (`L0.5`):** $O(N \log N)$ lazy-greedy max-heap submodular selection + `logos` DFA, `~300,000–400,000 ns` median (`~75–100 ns/byte`, `~412,000 ns` combined L0+L0.5 median), `< 1,500,000 ns` P99 ceiling per 4KB (Anti-Compression-Bomb).
   - **Tier 3 (`L1`):** `< 100,000 ns` (`~10–25 ns/byte`) Stage 3A-Fast $O(N)$ SIMD structural injection/exfiltration breaker + `~1,500,000–3,500,000 ns` median (`< 5,000,000 ns` hard `guard_timeout_ns` P99 ceiling across top-$K \le 2$ surprisal-ranked 512-token overlapping windows, preventing Neural Sponge CPU exhaustion).

A full cross-artifact audit identified **6 synchronization discrepancies** between the finalized [epics.md](file:///c:/Users/reidl/GitLocal/context-sanitizer/_bmad-output/planning-artifacts/epics.md) and the upstream planning/repository documents.

---

## 2. Impact Analysis

| Artifact / Area | Impact Assessment | Status |
| :--- | :--- | :--- |
| **[epics.md](file:///c:/Users/reidl/GitLocal/context-sanitizer/_bmad-output/planning-artifacts/epics.md)** | Authoritative baseline (5 Epics, 16 Stories, `FR-1`..`FR-14`, `NFR-1`..`NFR-8`, `AD-1`..`AD-6`, all latency budgets in `ns`). No structural changes required. | Verified (`status: final`) |
| **[prd.md](file:///c:/Users/reidl/GitLocal/context-sanitizer/_bmad-output/planning-artifacts/prds/prd-context-sanitizer-2026-10-22/prd.md)** | Contained 6 residual `ms`/`microsecond` strings in narrative sections (`Vision`, `UJ-3`, `Glossary`, `Feature 1`, `Feature 2`, `MVP Scope Tier 3`) and a Rust trait signature mismatch (`GuardError`/`CompressError` instead of unified `SanitizerError`). | Synchronized (`status: final`) |
| **[addendum.md](file:///c:/Users/reidl/GitLocal/context-sanitizer/_bmad-output/planning-artifacts/prds/prd-context-sanitizer-2026-10-22/addendum.md)** | Referenced `[REDACTED_*]` instead of `[REDACTED_CREDENTIAL]` and lacked `FR-14` / `AD-6` DevSecOps pipeline notes and nanosecond Anti-Sponge bounds. | Synchronized |
| **[ARCHITECTURE-SPINE.md](file:///c:/Users/reidl/GitLocal/context-sanitizer/_bmad-output/planning-artifacts/architecture/architecture-context-sanitizer-2026-10-22/ARCHITECTURE-SPINE.md)** | `Capability -> Architecture Map` row for `FR-8` needed explicit binding to the `< 100,000 ns` fast-breaker and `< 5,000,000 ns` `guard_timeout_ns` top-$K$ surprisal window rule. | Synchronized (`status: final`) |
| **[brief.md](file:///c:/Users/reidl/GitLocal/context-sanitizer/_bmad-output/planning-artifacts/briefs/brief-context-sanitizer-2026-10-22/brief.md)** | Frontmatter was `status: review`, used legacy `ms` thresholds, `[REDACTED_*]`, and listed Wasm (`FR-13`) and Turnkey Middleware (`FR-11`) under 2–3 Year Vision instead of v1.0 MVP Scope. | Synchronized (`status: final`) |
| **[prfaq-context-sanitizer.md](file:///c:/Users/reidl/GitLocal/context-sanitizer/_bmad-output/planning-artifacts/prfaq-context-sanitizer.md) & [prfaq-context-sanitizer-distillate.md](file:///c:/Users/reidl/GitLocal/context-sanitizer/_bmad-output/planning-artifacts/prfaq-context-sanitizer-distillate.md)** | Referenced legacy `ms` thresholds (`< 0.5 ms`, `< 2 ms`, `15–25 ms`) and `[REDACTED_*]` in FAQs and Distillate summary. | Synchronized (`status: complete`) |
| **[README.md](file:///c:/Users/reidl/GitLocal/context-sanitizer/README.md)** | Used `[REDACTED_*]` / `[REDACTED_AWS_KEY]` instead of `[REDACTED_CREDENTIAL]`, `pip install context-sanitizer` instead of `pip install context-sanitizer-py`, and omitted the `destination` whitelist parameter on `.rehydrate()`. | Synchronized |

---

## 3. Recommended Approach Evaluation

- **Selected Path:** **Option 1 — Direct Adjustment / Cross-Artifact Synchronization** (no scope reduction, no rollback, no MVP re-scoping required).
- **Rationale:** All 14 Functional Requirements (`FR-1` to `FR-14`), 8 Non-Functional Requirements (`NFR-1` to `NFR-8`), and 6 Architecture Decisions (`AD-1` to `AD-6`) were already fully decomposed and verified in [epics.md](file:///c:/Users/reidl/GitLocal/context-sanitizer/_bmad-output/planning-artifacts/epics.md). Bringing the upstream documents and `README.md` into exact lexical and contract alignment eliminates ambiguity for downstream sprint execution and automated coding agents.

---

## 4. Detailed Change Log Applied (Batch Mode)

1. **[prd.md](file:///c:/Users/reidl/GitLocal/context-sanitizer/_bmad-output/planning-artifacts/prds/prd-context-sanitizer-2026-10-22/prd.md):**
   - Updated Section 1 (`Vision`), Section 2.3 (`UJ-3`), Section 3 (`Glossary`), Section 4.1 (`Feature 1`), Section 4.2 (`Feature 2`), and Section 6.1 (`MVP Scope Tier 3`) to express all latencies in nanoseconds (`ns`).
   - Unified Section 8.3 (`Developer Product Contracts`) Rust trait signatures (`SemanticGuard::evaluate_threat` and `TokenCompressor::compress`) with [ARCHITECTURE-SPINE.md](file:///c:/Users/reidl/GitLocal/context-sanitizer/_bmad-output/planning-artifacts/architecture/architecture-context-sanitizer-2026-10-22/ARCHITECTURE-SPINE.md#L240-L248) using `SanitizerError`.
2. **[ARCHITECTURE-SPINE.md](file:///c:/Users/reidl/GitLocal/context-sanitizer/_bmad-output/planning-artifacts/architecture/architecture-context-sanitizer-2026-10-22/ARCHITECTURE-SPINE.md):**
   - Updated `Capability -> Architecture Map` (`FR-8`) to explicitly reflect `FR-8: Two-Stage Nanosecond Threat Gatekeeper` (`< 100,000 ns` fast-breaker + top-$K \le 2$ surprisal window under `< 5,000,000 ns` `guard_timeout_ns`).
3. **[addendum.md](file:///c:/Users/reidl/GitLocal/context-sanitizer/_bmad-output/planning-artifacts/prds/prd-context-sanitizer-2026-10-22/addendum.md):**
   - Standardized root credential masking on `[REDACTED_CREDENTIAL]`.
   - Added nanosecond (`ns`) Anti-DoS / Anti-Sponge latency bounds, unified `SanitizerError` trait contracts, and the `FR-14` / `AD-6` 6-stage DevSecOps + `.pre-commit-config.yaml` handoff notes.
4. **[brief.md](file:///c:/Users/reidl/GitLocal/context-sanitizer/_bmad-output/planning-artifacts/briefs/brief-context-sanitizer-2026-10-22/brief.md):**
   - Promoted status from `review` to `final`.
   - Updated all latency figures and KPIs to nanoseconds (`ns`), standardized on `[REDACTED_CREDENTIAL]`, and moved `@context-sanitizer/wasm` (`FR-13`), turnkey agent middleware (`FR-11`), and the 6-stage DevSecOps pipeline (`FR-14`) into v1.0 In-Scope.
5. **[prfaq-context-sanitizer.md](file:///c:/Users/reidl/GitLocal/context-sanitizer/_bmad-output/planning-artifacts/prfaq-context-sanitizer.md) & [prfaq-context-sanitizer-distillate.md](file:///c:/Users/reidl/GitLocal/context-sanitizer/_bmad-output/planning-artifacts/prfaq-context-sanitizer-distillate.md):**
   - Updated Customer/Internal FAQs, Verdict, and Distillate to use `[REDACTED_CREDENTIAL]`, destination-whitelisted `.rehydrate(args, destination="...")`, and nanosecond (`ns`) latency SLAs.
6. **[README.md](file:///c:/Users/reidl/GitLocal/context-sanitizer/README.md):**
   - Standardized all redaction tokens on `[REDACTED_CREDENTIAL]`.
   - Updated Python install command to `pip install context-sanitizer-py` and aligned Python/Rust code snippets with `cs.sanitize(...)`, `LangChainSanitizer`, and destination-whitelisted `vault.rehydrate(args, destination="...")`.

---

## 5. Implementation Handoff Plan

- **Scope Classification:** **Minor / Direct Artifact Synchronization** — all planning artifacts (`brief.md`, `prfaq-context-sanitizer.md`, `prfaq-context-sanitizer-distillate.md`, `prd.md`, `addendum.md`, `ARCHITECTURE-SPINE.md`, `epics.md`) and `README.md` are now 100% consistent.
- **Next Recommended Workflow:** Run **Sprint Planning** (`/bmad-sprint-planning`) to generate the sprint tracking file (`sprint-status.yaml`) from the finalized 5 Epics and 16 Stories in [epics.md](file:///c:/Users/reidl/GitLocal/context-sanitizer/_bmad-output/planning-artifacts/epics.md) and check implementation readiness.
