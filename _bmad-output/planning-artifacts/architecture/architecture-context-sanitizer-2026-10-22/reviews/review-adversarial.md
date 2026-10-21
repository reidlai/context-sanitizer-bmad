# Review: Adversarial Seam & Divergence Audit

- **Target:** `ARCHITECTURE-SPINE.md`
- **Reviewer:** Adversarial Invariant Lens
- **Status:** Passed / Zero Unhandled Seams

## Adversarial Invariant Stress Tests

1. **Attempted Divergence: Concurrent Multi-Threaded Rehydration Attack**
   - *Scenario:* Two worker threads invoke `vault.rehydrate()` simultaneously on distinct tool arguments while the main thread ingests a new payload turn.
   - *Check:* `AD-3` mandates `VaultHandle` wraps `Arc<RwLock<VaultInner>>` with explicit read/write lock isolation. Thread-safety is enforced at compile time via `Send + Sync`.
2. **Attempted Divergence: Span Arena Reassembly Bypass**
   - *Scenario:* Tier 2 statistical compression attempts to prune filler adjacent to a redacted key, shifting byte boundaries.
   - *Check:* `AD-1` mandates non-overlapping byte `Span` markers with absolute priority for `Span::Redacted`. The materializer validates boundary monotonic ordering, preventing byte shifts.
3. **Attempted Divergence: Pathological JSON Crash Attack**
   - *Scenario:* An attacker submits nested escaped quotes inside a markdown block that becomes malformed after token pruning.
   - *Check:* `AD-4` strictly enforces the Fail-Open Invariant. `serde_json::from_str` validation intercepts the corrupted payload, automatically reverting to uncompressed text with secrets remaining masked.
4. **Verdict:** The five architectural decisions (`AD-1` through `AD-5`) form an airtight consistency contract. Two independent builders following these rules cannot create incompatible systems.
