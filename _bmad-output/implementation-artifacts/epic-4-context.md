# Epic 4 Context: Zero-C Python Wheel & Turnkey Agent Framework Middleware (Python E2E)

<!-- Generated from planning artifacts. Regenerate with compile-epic-context if planning docs change. -->

## Goal

Python AI and systems engineers can install `context-sanitizer-py` across Linux, macOS, and Windows without a C/C++ or Rust compiler toolchain, dropping boundary secret redaction, cryptographic PII shadow vaulting, destination-whitelisted tool argument rehydration, and context compression into existing OpenAI, LangChain, CrewAI, and AutoGen agent workflows in two lines of code with $< 50,000\text{ ns}$ FFI overhead and GIL-free concurrency, verified via automated `maturin` wheel packaging and post-build black-box `pytest` CI gates.

## Stories

- Story 4.1: PyO3 Zero-C Core Bindings, GIL-Free Nanosecond FFI & Opaque `VaultHandle` Security
- Story 4.2: Turnkey Agent Framework Middleware (`LangChainSanitizer`, `CrewAISanitizer`, `AutoGenSanitizer`)
- Story 4.3: Zero-C Maturin Wheel Build Matrix & Clean-Virtualenv Post-Build E2E Integration Suite

## Requirements & Constraints

- **Zero-C Precompiled Wheel Distribution:** Distribute `context-sanitizer-py` as precompiled binary wheels built via `maturin` (`1.7.4`) targeting `manylinux_2_28`, `macos-universal2`, and `windows-x64`. `pip install context-sanitizer-py` must install cleanly with a 100% pass rate in clean virtual environments without requiring `gcc`, `clang`, MSVC, or a Rust toolchain.
- **Nanosecond FFI & GIL Management:** Expose `sanitize()` and `VaultHandle` via `pyo3` (`0.23.0`). The PyO3 boundary must release the Python GIL (`py.allow_threads(...)`) throughout Rust pipeline execution to ensure unblocked multi-threaded and asyncio concurrency, adding $< 50,000\text{ ns}$ FFI overhead per 4KB payload.
- **Opaque Vault Handle Security & Leak Prevention:**
  - Python `VaultHandle` wraps the Rust cryptographic session vault without exposing internal bidirectional hash maps, raw plaintext tokens, or 128-bit session salt bytes to Python runtime inspection.
  - `repr(vault)` and `str(vault)` must return safe diagnostic strings (`<VaultHandle session_active=True>`).
  - Access to `vault.__dict__` or serialization via `pickle` or `json` must fail explicitly with `TypeError`, preventing inadvertent PII leakage into exception tracebacks, APM logs, or telemetry dumps.
- **Destination-Whitelisted Tool Argument Rehydration:** Provide `vault.rehydrate(tool_args, destination=...)` where tool arguments (dict or JSON string) have `<SHADOW:...>` tokens replaced with original PII values if and only if `destination` matches the session whitelist. Unwhitelisted rehydration attempts must raise a typed `UnauthorizedRehydrationError`.
- **Turnkey Framework Middleware:**
  - Export two-line drop-in middleware wrappers from `context_sanitizer.integrations`: `LangChainSanitizer`, `CrewAISanitizer`, and `AutoGenSanitizer`.
  - Middleware intercepts outgoing message lists and `"tools"` schemas, applies the configured sanitization cascade before LLM dispatch, aborts with a security exception if an injection attack is intercepted, and automatically calls `vault.rehydrate()` on authorized local tool calls prior to tool execution.
- **DevSecOps Quality & CI Verification:**
  - Python code must pass formatting and linting (`ruff format --check`, `ruff check`), static security analysis (Semgrep / CodeQL), and dependency auditing (`pip-audit`).
  - Pre-build test gates require $\ge 85\%$ line coverage across both Rust bindings (`cargo llvm-cov --workspace --fail-under-lines 85`) and Python middleware (`pytest --cov=context_sanitizer --cov-fail-under=85`).
  - The PyO3 extension crate must feature-gate `pyo3/extension-module` so that `cargo test --workspace` compiles and links cleanly without requiring external Python shared libraries during native Rust CI runs.
  - Post-build black-box verification must provision a clean virtualenv with compilers disabled, install the built wheel, and execute the end-to-end multi-turn agent test suite (`tests/e2e_python_wheel.py`).

## Technical Decisions

- **Crate & Package Structure:**
  - Sub-crate path: `crates/context-sanitizer-py/`.
  - Workspace integration: Registered in root `Cargo.toml`. `pyproject.toml` configures the `maturin` build backend with module name `context_sanitizer._core`.
  - Python package layout:
    - `crates/context-sanitizer-py/src/lib.rs`: PyO3 module exporting `sanitize`, `VaultHandle`, and typed exceptions.
    - `crates/context-sanitizer-py/python/context_sanitizer/`: Python package root.
    - `crates/context-sanitizer-py/python/context_sanitizer/__init__.py`: Exposes top-level `sanitize`, `VaultHandle`, and exceptions.
    - `crates/context-sanitizer-py/python/context_sanitizer/integrations/`: Turnkey agent wrappers (`langchain.py`, `crewai.py`, `autogen.py`).
- **PyO3 Feature Gating:** Gate `extension-module` behind an optional Cargo feature or pass it during `maturin build` so that `cargo test --workspace` links cleanly in native CI without `undefined symbol: PyExc_TypeError` link errors.
- **Unidirectional Hexagonal Boundary:** `context-sanitizer-py` depends unidirectionally on `context-sanitizer` (Core). Core remains pure Rust with zero PyO3, Python, or C runtime dependencies.
- **Exception Hierarchy:** Map Rust `SanitizerError` variants to Python exceptions:
  - `SanitizerError::InjectionBlocked` $\to$ `InjectionBlockedError` (exposing threat score and reason).
  - `SanitizerError::UnauthorizedRehydration` $\to$ `UnauthorizedRehydrationError`.
  - Other engine errors $\to$ `SanitizerError`.
- **Payload Ingestion & Materialization:** Accept Python dicts, lists of messages, or JSON strings. Deserialize into the Core wire model, execute pipeline passes, and materialize sanitized structures and the session `VaultHandle` with minimal conversion overhead.

## UX & Interaction Patterns

- **Direct Python API (`import context_sanitizer as cs`):**
  ```python
  import context_sanitizer as cs

  # Sanitize outgoing messages
  sanitized_messages, vault = cs.sanitize(
      messages,
      rate=0.6,
      whitelist=["stripe_lookup"]
  )

  # Rehydrate arguments before invoking authorized internal tool
  real_args = vault.rehydrate(tool_call.arguments, destination="stripe_lookup")
  ```
- **Turnkey Agent Middleware:**
  ```python
  from context_sanitizer.integrations import LangChainSanitizer

  # Wrap model or pipeline in two lines
  sanitizer = LangChainSanitizer(rate=0.6, whitelist=["stripe_lookup"])
  protected_chain = sanitizer.wrap(llm)
  ```
- **Security Invariant Verification:**
  - `repr(vault)` returns `<VaultHandle session_active=True>` (or `<VaultHandle session_active=False>` once invalidated).
  - `pickle.dumps(vault)` raises `TypeError("VaultHandle cannot be pickled")`.

## Cross-Story Dependencies

- **Story 4.1 $\to$ Stories 4.2 & 4.3:** Story 4.1 establishes the core PyO3 module (`context_sanitizer._core`), `sanitize()`, `VaultHandle`, and exception bindings that Story 4.2 integrates into agent frameworks and Story 4.3 packages into release wheels.
- **Story 4.2 $\to$ Story 4.3:** The agent framework middleware modules developed in Story 4.2 are validated by the clean-virtualenv post-build E2E test suite in Story 4.3.
- **Epics 1, 2, & 3 $\to$ Epic 4:**
  - Epic 1 provides the base `Span` arena, `VaultHandle`, wire models, and `SanitizerError`.
  - Epic 2 provides statistical token compression (`rate` parameter) and JSON syntax validation.
  - Epic 3 provides threat gatekeeper classification and `InjectionBlocked` exceptions.
- **Epic 4 $\to$ Epic 5:** Epic 4 establishes the multi-platform packaging, pre/post-build CI gates, and consumer wrapper paradigms leveraged for the universal WebAssembly module in Epic 5 and the final release pipeline (`release.yml`).
