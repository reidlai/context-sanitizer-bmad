# context-sanitizer-bmad

[![Buy Me a Coffee](https://img.shields.io/badge/Buy%20Me%20a%20Coffee-reidlai-orange?logo=buy-me-a-coffee)](https://www.buymeacoffee.com/reidlai)

This repository houses the BMad planning, design artifacts, agent personas, and development specifications for `context-sanitizer`.

## Repository & Submodule Structure

This project uses a two-repo submodule layout:

- **Root Repository (`context-sanitizer-bmad`)**:
  - `_bmad-output/`: Planning documents, architecture decision records (ADRs), user stories, PRD, and implementation artifacts.
  - `.agent/`: BMad agent definitions, skills, and elicitation workflows.
  - Submodule pointer to `context-sanitizer/`.

- **Core Submodule (`context-sanitizer/`)**:
  - Located at `context-sanitizer/` (git submodule tracking active feature branches, e.g. `feat/epic-2`).
  - Contains all pure-Rust production code, crates, tests, and CI workflows:
    - `crates/context-sanitizer`: Core library providing Tier 1 deterministic secret redaction, operational PII shadow vaulting, Tier 2 syntax-safe statistical context compression, and OpenAI wire transformers.
    - `crates/context-sanitizer-cli`: Standalone `context-sanitizer` binary providing the `xray` latency dashboard and automated benchmarking harness.
    - `.github/workflows/ci.yml`: 6-stage DevSecOps CI pipeline.

## Quick Start

### Cloning with Submodules

```bash
git clone --recurse-submodules https://github.com/reidlai/context-sanitizer-bmad.git
cd context-sanitizer-bmad
```

If already cloned:

```bash
git submodule update --init --recursive
```

### Building and Testing

All Rust development is executed inside the submodule:

```bash
cd context-sanitizer
cargo build
cargo test --workspace
cargo test --release --test e2e_tier2_compression
cargo run --bin context-sanitizer -- xray --help
```
