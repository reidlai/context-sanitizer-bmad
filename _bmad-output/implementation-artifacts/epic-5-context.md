# Epic 5 Context: Universal WebAssembly, Edge Client Sanitization & Multi-Target Release Pipeline (Wasm + Release E2E)

<!-- Generated from planning artifacts. Regenerate with compile-epic-context if planning docs change. -->

## Goal

Frontend and edge engineers can deploy `@context-sanitizer/wasm` (`wasm32-unknown-unknown` and `wasm32-wasip1`) inside browser extensions, Electron apps, Node.js, and Cloudflare Workers to perform client-side Tier 1 credential/PII sanitization, $< 100,000\text{ ns}$ lexical injection fast-breaking, and Tier 2 statistical compression in $< 1,000,000\text{ ns}$ median before payloads ever leave the client runtime, complete with the final multi-target DevSecOps release and provenance pipeline (`release.yml`) gating all release artifacts (CLI binaries, zero-C Python wheels, Wasm packages, SBOMs, and SHA-256 checksums) behind passing pre- and post-build verification across Linux, macOS, and Windows.

## Stories

- Story 5.1: Universal WebAssembly / WASI Bindings (`@context-sanitizer/wasm`), W3C Web Cryptography CSPRNG & Nanosecond Execution
- Story 5.2: Post-Build Wasm/WASI E2E Integration Suite & Multi-Target DevSecOps Release Pipeline (`release.yml`)

## Requirements & Constraints

- **Universal WebAssembly & WASI Compilation:**
  - Compile the `wasm/` crate (`@context-sanitizer/wasm`) via `wasm-pack build` for `wasm32-unknown-unknown` (`web`, `bundler`, `nodejs`) and `cargo build --target wasm32-wasip1`.
  - Maintain pure Rust execution with zero C/C++ toolchain dependencies, zero native OS syscall imports, and `#![forbid(unsafe_code)]` preserved in Core.
  - Compile with default zero-ML features: Tier 1 credential/PII guard, Stage 3A-Fast lexical breaker, and Tier 2 statistical compressor.
- **W3C Web Cryptography & WASI CSPRNG Entropy:**
  - The 128-bit session salt (`[u8; 16]`) must source cryptographic entropy strictly from CSPRNG algorithms: W3C Web Cryptography API (`crypto.getRandomValues`) on `wasm32-unknown-unknown` and WASI `random_get` on `wasm32-wasip1`.
  - Never fall back to predictable `Math.random()` or static seeds.
  - Session deallocation via `.free()` or JavaScript garbage collection (`FinalizationRegistry`) must immediately drop in-memory vault state and session salt from Wasm linear memory.
- **Nanosecond Performance SLA:**
  - Client-side Tier 1 + Tier 2 pipeline execution must complete in $< 1,000,000\text{ ns}$ median ($< 2,500,000\text{ ns}$ P99 on adversarial payloads) per 4KB payload in V8 and Wasmtime runtimes.
- **TypeScript & JavaScript Surface API:**
  - Export `sanitize(payloadJson, rate, whitelist)` accepting JSON strings or structured payloads, returning sanitized JSON and an opaque `WasmVaultHandle`.
  - Provide full TypeScript `.d.ts` declaration typings.
  - `WasmVaultHandle.rehydrate(toolArgs, destination)` must replace `<SHADOW:...>` tokens if and only if `destination` matches the session whitelist, raising a typed JavaScript error on unwhitelisted access attempts.
- **Post-Build Black-Box Wasm E2E Integration Suite:**
  - Stage 6 executes automated post-build integration tests (`wasm/tests/e2e_wasm_client.mjs`) directly against the compiled Wasm package in Node.js / headless browser and Wasmtime.
  - Verify complete workflow: (1) ingestion of OpenAI chat completion payloads containing root cloud keys, customer PII (emails, IPv4/IPv6, phone numbers), code blocks, and `"tools"` schemas; (2) 100% root credential redaction (`[REDACTED_CREDENTIAL]`); (3) cryptographic shadow token isolation; (4) 40%–70% token compression with 0% JSON parse errors (fail-open syntax invariant); and (5) authorized vs. unauthorized `rehydrate()` calls.
- **Automated DevSecOps Release & Provenance Pipeline (`release.yml`):**
  - Implement `.github/workflows/release.yml` triggered on version tags (`v*`) or manual workflow dispatch.
  - Block publishing until all 6 DevSecOps stages pass across Linux, macOS, and Windows:
    1. Formatting & Linting (`cargo fmt`, `clippy`, `ruff`).
    2. SAST & Secret Scanning (`#![forbid(unsafe_code)]`, panic-free lints, Semgrep/CodeQL, `gitleaks`).
    3. SCA & License Auditing (`cargo-audit`, `cargo-deny check`, `pip-audit`).
    4. Pre-Build Unit & Source Integration Tests + $\ge 85\%$ Coverage Gate (`cargo-llvm-cov` and `pytest-cov`).
    5. Multi-Target Release Compilation (CLI binaries, `maturin` wheels for `manylinux_2_28`/`macos-universal2`/`windows-x64`, and `@context-sanitizer/wasm`).
    6. Post-Build Black-Box Artifact E2E Integration Tests & SLA Benchmark Gates.
  - Generate CycloneDX/SPDX Software Bill of Materials (SBOMs) and SHA-256 checksum manifests alongside all published release artifacts.

## Technical Decisions

- **Crate Layout & Workspace Registration:**
  - Sub-crate directory: `wasm/`.
  - Workspace integration: Add `wasm` to root `Cargo.toml` workspace members in Epic 5 to maintain $\ge 85\%$ workspace coverage integrity.
  - Configuration: `wasm/Cargo.toml` with `crate-type = ["cdylib", "rlib"]`, depending unidirectionally on `crates/context-sanitizer` (Core) with default features.
  - Package metadata: `wasm/package.json` for `@context-sanitizer/wasm` with exports for `bundler`, `web`, and `node` targets.
  - Pinned binding dependency: `wasm-bindgen = "0.2.95"`.
- **Unidirectional Hexagonal Boundary:**
  - `wasm` depends unidirectionally on `context-sanitizer` (Core).
  - Core remains strictly platform-agnostic with zero dependencies on `wasm-bindgen`, browser globals, or Node.js APIs.
- **Target-Conditional CSPRNG Hook:**
  - Enable `getrandom` with the `js` feature for `wasm32-unknown-unknown` to bridge to `globalThis.crypto.getRandomValues`.
  - Use native WASI entropy `random_get` for `wasm32-wasip1`.
- **Exception & Error Mapping:**
  - Map `SanitizerError::InjectionBlocked` to JavaScript errors with `threat_score` and `reason` metadata.
  - Map `SanitizerError::UnauthorizedRehydration` to typed JavaScript `UnauthorizedRehydrationError`.
  - Map engine failures to standard `Error` with descriptive diagnostics.
- **Multi-Target Release Matrix (`release.yml`):**
  - CLI binaries: Linux (`x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`), macOS (`x86_64-apple-darwin`, `aarch64-apple-darwin`), Windows (`x86_64-pc-windows-msvc`).
  - Python wheels: `manylinux_2_28`, `macos-universal2`, `windows-x64`.
  - WebAssembly: `@context-sanitizer/wasm` bundle and `wasm32-wasip1` binary.
  - Provenance: Automated `syft` or `cargo-cyclonedx` SBOM generation and `SHA256SUMS` creation.

## UX & Interaction Patterns

- **In-Browser / Edge Client Sanitization (TypeScript):**
  ```typescript
  import { sanitize } from "@context-sanitizer/wasm";

  // Sanitize raw payload client-side before network transmission
  const { sanitizedJson, vault } = sanitize(
    JSON.stringify(openAiPayload),
    0.6,
    ["stripe_lookup"]
  );

  // Send sanitizedJson to external LLM endpoint
  const response = await fetch("https://api.openai.com/v1/chat/completions", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: sanitizedJson
  });

  // Rehydrate arguments locally before executing authorized tool
  const safeArgs = vault.rehydrate(toolCall.arguments, "stripe_lookup");

  // Explicit session cleanup
  vault.free();
  ```
- **Error Handling Pattern:**
  ```typescript
  try {
    const { sanitizedJson, vault } = sanitize(payload);
  } catch (err) {
    if (err.name === "InjectionBlockedError") {
      console.warn(`Injection blocked: ${err.reason} (score: ${err.score})`);
    }
  }
  ```

## Cross-Story Dependencies

- **Story 5.1 $\to$ Story 5.2:** Story 5.1 builds the `wasm/` crate, `wasm-bindgen` bindings, CSPRNG entropy integration, and TypeScript typings that Story 5.2 validates via the black-box E2E test suite and distributes via `release.yml`.
- **Epics 1, 2, 3, & 4 $\to$ Epic 5:**
  - Epic 1 provides core `SpanArena`, `VaultHandle`, credential redaction, and PII vaulting.
  - Epic 2 provides syntax token locking, statistical compression, and schema minification.
  - Epic 3 provides Stage 3A-Fast lexical breaker for injection defense.
  - Epic 4 provides precompiled packaging conventions and clean-environment black-box testing patterns.
  - Epic 5 unifies all build artifacts from Epics 1 through 5 (CLI, Python wheels, Wasm packages) under the automated 6-stage DevSecOps release pipeline (`release.yml`).
