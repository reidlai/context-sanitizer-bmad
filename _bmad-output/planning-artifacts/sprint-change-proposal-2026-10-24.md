# Sprint Change Proposal: Multi-Registry Publishing & Hardened Supply-Chain Release Pipeline (Epic 6)

**Date:** 2026-10-24  
**Trigger:** Distribution readiness for workspace crates (`context-sanitizer`, `context-sanitizer-cli`, `context-sanitizer-py`) and the October 2026 eMudhra/emSign TLS Certificate Authority compromise affecting PyPI package delivery and supply-chain integrity.  
**Author:** Developer Agent (Amelia)  
**Status:** Proposed  

---

## 1. Issue Summary

### 1.1 Context
With the core implementation of Epics 1 through 5 complete and verified (deterministic secret redaction, statistical token compression, System One threat gatekeeper, PyO3 Python bindings, and WebAssembly compilation), the project requires a production-grade multi-registry publication pipeline to enable seamless consumer installation:
- `cargo add context-sanitizer` for Rust library consumers.
- `cargo install context-sanitizer-cli` / GitHub Release binaries for terminal diagnostics.
- `pip install context-sanitizer-py` for Python and AI agent framework developers.

### 1.2 Supply-Chain Trigger: PyPI TLS Certificate Authority Compromise
In late October 2026, a severe supply-chain security event was disclosed concerning the **eMudhra/emSign Certificate Authority (CA)**. Private keys for eight intermediate TLS certificates were discovered to have been generated with critically low entropy, enabling trivial key recovery and the potential for malicious actors to forge trusted SSL/TLS certificates and perform Man-in-the-Middle (MitM) attacks against HTTPS connections.

Python package distribution is uniquely susceptible because:
1. `pip` vendors its own copy of the `certifi` root CA bundle rather than checking the operating system trust store or performing real-time CRL/OCSP revocation checks.
2. Unpatched build environments and outdated package upload tools can be tricked by forged certificates or leak static API tokens over intercepted TLS channels.

To ensure `context-sanitizer-py` is published securely without risk of token interception, payload tampering, or supply-chain compromise, the release pipeline must incorporate immediate CA remediations, trusted OIDC publishing, and Sigstore provenance attestations.

---

## 2. Impact Analysis

### 2.1 Epic Impact
- **Existing Epics (Epics 1–5):** Status remains `done`. No regressions or architectural rollbacks required for core crates.
- **New Epic:** Add **Epic 6: Multi-Registry Publishing & Hardened Supply-Chain Release Pipeline**.

### 2.2 Story Impact
Add three new implementation stories under Epic 6:
- **Story 6.1:** Publish `context-sanitizer` Core Library to Crates.io.
- **Story 6.2:** Publish `context-sanitizer-cli` Binary to Crates.io & Multi-Arch GitHub Releases.
- **Story 6.3:** Hardened PyPI Publication for `context-sanitizer-py` with Supply-Chain TLS Mitigations.

### 2.3 Artifact Conflicts & Updates
- **`epics.md`:** Append Epic 6 and Stories 6.1, 6.2, 6.3.
- **`sprint-status.yaml`:** Register `epic-6` and three stories in `ready-for-dev`.
- **`Cargo.toml` (Core & CLI):** Ensure crate metadata (`readme`, `categories`, `keywords`, `documentation`, `repository`) conforms to crates.io distribution requirements.
- **`.github/workflows/release.yml`:** Enhance release workflows with OIDC Trusted Publishing, toolchain version pinning (`pip >= 2026.2.2`, `certifi >= 2026.10.19`, `uv >= 0.13.4`, `sigstore >= 4.5.1`), and cryptographic attestations.

---

## 3. Recommended Approach

**Direct Adjustment:** Expand the release phase by introducing **Epic 6** directly into the sprint plan without altering existing completed deliverables.

### Risk & Effort Assessment
- **Effort:** Low-to-Medium (configuration, metadata validation, CI workflow updates).
- **Risk:** Low risk of code regression; high security return by neutralizing supply-chain MitM and credential exfiltration vectors.
- **Timeline Impact:** None on core functionality; strengthens production readiness and open-source compliance.

---

## 4. Detailed Story Proposals

### Epic 6: Multi-Registry Publishing & Hardened Supply-Chain Release Pipeline

Enable developers worldwide to consume `context-sanitizer` through standard package managers (`cargo add context-sanitizer`, `cargo install context-sanitizer-cli`, `pip install context-sanitizer-py`) backed by automated, cryptographically signed, and TLS-hardened release pipelines.

---

#### Story 6.1: Publish `context-sanitizer` Core Library to Crates.io

**As a** Rust backend developer,  
**I want** `context-sanitizer` published to crates.io with complete documentation and package metadata,  
**So that** I can add it to my project using `cargo add context-sanitizer` with zero manual configuration.

**Acceptance Criteria:**
1. **Given** [`crates/context-sanitizer/Cargo.toml`](file:///c:/Users/reidl/GitLocal/context-sanitizer-bmad/context-sanitizer/crates/context-sanitizer/Cargo.toml)
   **When** validated for crates.io packaging
   **Then** it specifies `readme = "../../README.md"`, `categories = ["development-tools", "parsing", "web-programming"]`, `keywords = ["sanitizer", "llm", "pii", "token-compression", "prompt-security"]`, and `documentation = "https://docs.rs/context-sanitizer"`.
2. **Given** a git release tag (`v*.*.*`) pushed to the repository
   **When** `.github/workflows/release.yml` executes Stage 5
   **Then** it runs `cargo publish -p context-sanitizer --dry-run` to verify crate integrity before executing the final `cargo publish -p context-sanitizer` using the scoped crates.io API secret (`CRATES_IO_TOKEN`).
3. **Given** the published crate on crates.io
   **When** a downstream consumer executes `cargo add context-sanitizer`
   **Then** Cargo installs version `0.1.0` and resolves all required pure-Rust dependencies (`logos`, `aho-corasick`, `serde`, `thiserror`) with zero C-toolchain requirements.

---

#### Story 6.2: Publish `context-sanitizer-cli` Binary to Crates.io & Multi-Arch GitHub Releases

**As a** DevOps engineer or security analyst,  
**I want** `context-sanitizer-cli` published to crates.io and distributed as precompiled multi-architecture release binaries on GitHub Releases,  
**So that** I can install `context-sanitizer` globally via `cargo install context-sanitizer-cli` or download standalone executables for CI/CD pipelines without compiling from source.

**Acceptance Criteria:**
1. **Given** [`crates/context-sanitizer-cli/Cargo.toml`](file:///c:/Users/reidl/GitLocal/context-sanitizer-bmad/context-sanitizer/crates/context-sanitizer-cli/Cargo.toml)
   **When** prepared for crates.io publishing
   **Then** the package metadata (`description`, `readme`, `keywords`, `categories`, `repository`) is validated, and `context-sanitizer` dependency points to the published crates.io version `0.1.0`.
2. **Given** a release workflow run
   **When** building standalone CLI executables
   **Then** it compiles optimized binaries across target matrix:
     - `x86_64-unknown-linux-gnu` / `x86_64-unknown-linux-musl`
     - `aarch64-unknown-linux-gnu`
     - `x86_64-apple-darwin` / `aarch64-apple-darwin`
     - `x86_64-pc-windows-msvc`
   **And** generates `.tar.gz` and `.zip` archives with SHA-256 checksums uploaded to the GitHub Release.
3. **Given** an environment with standard Cargo installed
   **When** executing `cargo install context-sanitizer-cli`
   **Then** the `context-sanitizer` binary installs into Cargo's bin directory and supports `context-sanitizer xray --help` and `context-sanitizer xray --bench`.

---

#### Story 6.3: Hardened PyPI Publication for `context-sanitizer-py` with Supply-Chain TLS Mitigations

**As an** AI agent engineer and DevSecOps lead,  
**I want** `context-sanitizer-py` built and published to PyPI using strict TLS certificate verification, minimum patched tooling (`pip >= 2026.2.2`, `certifi >= 2026.10.19`, `uv >= 0.13.4`), PyPI OIDC Trusted Publishing, and Sigstore provenance attestations,  
**So that** downstream Python users are completely protected from the October 2026 eMudhra/emSign TLS CA compromise and supply-chain MitM attacks.

**Acceptance Criteria:**
1. **Given** the CI release job for Python wheel distribution
   **When** configuring the build and publishing environments
   **Then** all workflow runners explicitly upgrade and pin:
     - `pip >= 2026.2.2` (removing the compromised eMudhra/emSign intermediate CAs from the vendored bundle)
     - `certifi >= 2026.10.19`
     - `uv >= 0.13.4`
     - `sigstore >= 4.5.1`
   **And** run `update-ca-certificates` on Linux runners to purge outdated system certificate caches.
2. **Given** wheel publication to PyPI
   **When** authenticating against `upload.pypi.org`
   **Then** the workflow utilizes **PyPI Trusted Publishing (OIDC)** via `pypa/gh-action-pypi-publish` with `id-token: write` permissions, eliminating static PyPI API tokens that could otherwise be intercepted across compromised TLS intermediaries.
3. **Given** built `.whl` artifacts across Linux (`manylinux_2_28`), macOS (`macos-universal2`), and Windows (`windows-x64`)
   **When** uploaded to PyPI
   **Then** the pipeline generates **Sigstore cryptographic signatures** and SLSA provenance attestations (`attestations: true`), enabling downstream clients to cryptographically verify package integrity and publisher authenticity.
4. **Given** a clean consumer environment
   **When** executing `pip install context-sanitizer-py`
   **Then** the wheel installs cleanly without C/C++ compilers and passes `python -c "import context_sanitizer as cs; print(cs.__version__)"`.

---

## 5. Implementation Handoff

- **Change Scope:** Moderate (Sprint Backlog Extension).
- **Handoff Recipient:** Developer Agent (Amelia).
- **Deliverables:**
  1. Updated [`epics.md`](file:///c:/Users/reidl/GitLocal/context-sanitizer-bmad/_bmad-output/planning-artifacts/epics.md) and [`sprint-status.yaml`](file:///c:/Users/reidl/GitLocal/context-sanitizer-bmad/_bmad-output/implementation-artifacts/sprint-status.yaml).
  2. Cargo metadata enhancements in `crates/context-sanitizer/Cargo.toml` and `crates/context-sanitizer-cli/Cargo.toml`.
  3. Hardened release automation in `.github/workflows/release.yml`.
