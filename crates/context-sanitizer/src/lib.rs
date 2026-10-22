#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! # `context-sanitizer`
//!
//! Pure-Rust **System One semantic firewall**, **Split-Secret Shadow Vault**, and
//! **syntax-safe token compressor** for Agentic AI, Copilot UIs, and OpenAI-compatible boundaries.
//!
//! ## Architecture Invariants
//! - **AD-1 (Span-Annotated Pipes-and-Filters Cascade):** Zero-copy, read-only byte span passes
//!   where `Redacted` and `Shadow` spans hold strict priority over `Locked`, `Pruned`, and `Retained` spans.
//! - **AD-2 (Pure-Rust Core Isolation):** `#![forbid(unsafe_code)]` with zero external C/C++ dependencies.
//! - **AD-3 (Session-Scoped Thread-Safe Vault):** Cryptographically isolated `<SHADOW:<TYPE>_<HEX6>>` tokens
//!   with destination-whitelisted `.rehydrate(args, destination)`.
//! - **AD-4 (Fail-Open Syntax Verification):** Immutable lexical locking of JSON/code delimiters with
//!   fail-open fallback guaranteeing `0%` downstream JSON parse errors.

use thiserror::Error;

/// Unified error type across the `context-sanitizer` pipeline (`AD-2`).
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SanitizerError {
    /// Raised when a syntax lock invariant is violated during token pruning.
    #[error("syntax lock violation: {0}")]
    SyntaxLockViolation(String),

    /// Raised when `.rehydrate()` is invoked for an unregistered or external destination (`FR-3`).
    #[error("unauthorized rehydration destination: {0}")]
    UnauthorizedRehydration(String),

    /// Raised when the System One threat gatekeeper blocks a prompt injection or exfiltration attempt (`FR-8`).
    #[error("threat detected (score: {score_bps} bps): {reason}")]
    ThreatBlocked {
        /// Threat score in basis points (0..=10000).
        score_bps: u16,
        /// Human-readable reason for the block.
        reason: String,
    },
}

/// Non-overlapping byte span annotation over an immutable input payload (`AD-1`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    /// Inclusive start byte offset (UTF-8 char boundary).
    pub start: usize,
    /// Exclusive end byte offset (UTF-8 char boundary).
    pub end: usize,
    /// Classification and precedence of this span.
    pub kind: SpanKind,
}

/// Precedence-ordered span classifications (`Redacted` > `Shadow` > `Locked` > `Pruned` > `Retained`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpanKind {
    /// Irreversible one-way credential mask (`[REDACTED_CREDENTIAL]`).
    Redacted(&'static str),
    /// Session-scoped operational PII shadow token (`<SHADOW:<TYPE>_<HEX6>>`).
    Shadow(String),
    /// Immutable syntax token (JSON structural delimiter, code fence, shell substitution).
    Locked,
    /// Low-entropy token marked for removal during Tier 2 compression.
    Pruned,
    /// Token retained in the final materialized payload.
    Retained,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_construction_preserves_bounds() {
        let span = Span {
            start: 0,
            end: 21,
            kind: SpanKind::Redacted("[REDACTED_CREDENTIAL]"),
        };
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 21);
        assert_eq!(span.kind, SpanKind::Redacted("[REDACTED_CREDENTIAL]"));
    }
}
