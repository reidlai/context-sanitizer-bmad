# Epic 6 Context: Multi-Registry Publishing & Hardened Supply-Chain Release Pipeline

<!-- Generated from planning artifacts. Regenerate with compile-epic-context if planning docs change. -->

## Goal

Enable developers worldwide to consume `context-sanitizer` through standard package managers (`cargo add context-sanitizer`, `cargo install context-sanitizer-cli`, `pip install context-sanitizer-py`) backed by automated, cryptographically signed, and TLS-hardened release pipelines protected against the October 2026 eMudhra/emSign CA compromise.

## Stories

- Story 6.1: Publish `context-sanitizer` Core Library to Crates.io
- Story 6.2: Publish `context-sanitizer-cli` Binary to Crates.io & Multi-Arch GitHub Releases
- Story 6.3: Hardened PyPI Publication for `context-sanitizer-py` with Supply-Chain TLS Mitigations

## Requirements & Constraints

- **Crates.io Core Library Metadata (`context-sanitizer`)**: The core library crate must specify complete metadata (`readme`, `categories`, `keywords`, `documentation`, `repository`, `license`) and pass `cargo publish -p context-sanitizer --dry-run` with zero warnings before deployment.
- **Crates.io & Binary Releases (`context-sanitizer-cli`)**: The CLI crate must be publishable to Crates.io with `context-sanitizer` workspace dependency pointing to the published crate version `0.1.0`. Precompiled release binaries across 5 target architectures (`x86_64-linux` gnu/musl, `aarch64-linux`, `macos-universal2`, `windows-x64`) must be packaged into `.tar.gz`/`.zip` archives with SHA-256 checksum manifests for GitHub Releases.
- **PyPI Supply-Chain TLS Mitigations (`context-sanitizer-py`)**: Address the October 2026 eMudhra/emSign Certificate Authority compromise by enforcing:
  - Minimum patched tooling in all release runners: `pip >= 2026.2.2` (removes weak intermediate CAs from the vendored cert bundle), `certifi >= 2026.10.19`, `uv >= 0.13.4`, and `sigstore >= 4.5.1`.
  - Base container trust refresh (`update-ca-certificates`).
  - PyPI Trusted Publishing via OpenID Connect (OIDC `id-token: write` with `pypa/gh-action-pypi-publish`), removing static API tokens that can be intercepted via MitM proxies.
  - Digital provenance and Sigstore signatures (`attestations: true`) for all uploaded wheels.
- **Zero-C & Memory Safety Invariants (`AD-2`, `AD-6`)**: `#![forbid(unsafe_code)]` remains strictly enforced; wheels must compile cleanly with Maturin with zero external C/C++ runtime dependencies.

## Technical Decisions

- **Unidirectional Workspace Packaging**: `crates/context-sanitizer` (Core) has zero dependencies on surface crates. `crates/context-sanitizer-cli` and `crates/context-sanitizer-py` depend unidirectionally on the core crate.
- **Automated Two-Stage Release Workflow (`.github/workflows/release.yml`)**:
  - Gated behind full 6-stage DevSecOps checks (formatting, SAST, SCA, coverage $\ge 85\%$, artifact compilation, and black-box E2E testing).
  - Dry-run verification prior to final upload.
- **OIDC Identity Federation**: Secure token exchange directly between GitHub Actions and PyPI / crates.io, rendering network traffic interception attacks harmless.

## Cross-Story Dependencies

- **Story 6.1 precedes Story 6.2**: `context-sanitizer-cli` depends on `context-sanitizer`; publishing core to crates.io enables CLI to resolve its dependency in independent crates.io installations.
- **Story 6.3 runs in parallel**: `context-sanitizer-py` bundles the core statically into its Python C-extension (`cdylib`), so PyPI publishing can be verified independently via Maturin wheel matrix builds.
