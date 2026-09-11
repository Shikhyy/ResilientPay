//! Typed error hierarchy for the ResilientPay SDK core.
//!
//! Every security boundary returns a typed error. Callers are forced by the type
//! system to handle the distinction between:
//!
//! - a malformed input (`ValidationError`)
//! - an illegal state transition (`TransitionError`)
//! - a runtime failure that is not a security rejection (`CoreError`)
//!
//! # Design rule
//!
//! Never use bare `bool` returns or untyped `String` errors at security boundaries.
//! See `docs/06-development/CODING_STANDARDS.md` §Error semantics.

use thiserror::Error;

// ---------------------------------------------------------------------------
// Validation errors — malformed or policy-violating inputs
// ---------------------------------------------------------------------------

/// Errors produced when an input field or envelope fails validation.
///
/// These errors are safe to log (they contain no secret material).
/// They do NOT contain private keys, PINs, or raw credential bytes.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum ValidationError {
    /// The local offline budget is exceeded.
    #[error("offline budget exceeded: transaction amount {amount_minor} + outstanding {outstanding} exceeds limit {max_outstanding}")]
    OfflineBudgetExceeded { amount_minor: u64, outstanding: u64, max_outstanding: u64 },

    /// The monetary amount exceeds the configured ceiling.
    #[error("amount {amount_minor} exceeds maximum {max}")]
    AmountExceedsMaximum { amount_minor: u64, max: u64 },

    /// The currency code is invalid.
    #[error("invalid currency: {reason}")]
    InvalidCurrency { reason: String },

    /// The protocol version in the envelope is not supported by this implementation.
    #[error(
        "unsupported protocol version {version}; this implementation supports version {supported}"
    )]
    UnsupportedProtocolVersion { version: u32, supported: u32 },

    /// A required field is missing or empty.
    #[error("missing required field: {field}")]
    MissingField { field: String },

    /// A field value is outside its permitted range.
    #[error("field {field} value out of range: {reason}")]
    FieldOutOfRange { field: String, reason: String },

    /// The transaction has expired (based on `expires_at` metadata).
    #[error("transaction expired")]
    TransactionExpired,

    /// The nonce violates structural constraints (e.g., wrong length).
    #[error("invalid nonce: {reason}")]
    InvalidNonce { reason: String },

    /// The counter value violates the protocol's counter/replay rule.
    #[error("counter violation: {reason}")]
    CounterViolation { reason: String },

    /// A text field contains invalid or disallowed characters.
    #[error("field {field} contains invalid characters: {reason}")]
    InvalidCharacters { field: String, reason: String },
}

// ---------------------------------------------------------------------------
// State machine transition errors
// ---------------------------------------------------------------------------

/// Errors produced when a state transition is attempted from an invalid source state
/// or with an inapplicable event.
///
/// # Protocol reference
///
/// `docs/05-protocol/TRANSACTION_STATE_MACHINE.md` §2 (Transition rules)
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum TransitionError {
    /// The transition from `current` state with the given event is not permitted
    /// by the state machine specification.
    #[error("illegal transition: state {current_state} does not accept event {event}")]
    IllegalTransition {
        current_state: String,
        event: String,
    },

    /// A terminal state was reached; no further transitions are permitted.
    #[error("state {current_state} is terminal; no further transitions are possible")]
    TerminalState { current_state: String },
}

// ---------------------------------------------------------------------------
// Core operational errors
// ---------------------------------------------------------------------------

/// Top-level error type for SDK operations that combine validation and infrastructure.
#[derive(Debug, Error)]
pub enum CoreError {
    /// A validation check failed.
    #[error("validation failed: {0}")]
    Validation(#[from] ValidationError),

    /// A state machine transition failed.
    #[error("state transition failed: {0}")]
    Transition(#[from] TransitionError),

    /// An internal invariant was violated. This indicates a bug in the SDK,
    /// not a protocol error from external input.
    #[error("internal invariant violated: {reason}")]
    InternalInvariant { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_error_is_display_safe() {
        let e = ValidationError::AmountExceedsMaximum {
            amount_minor: 999,
            max: 100,
        };
        let display = e.to_string();
        assert!(display.contains("999"));
        assert!(display.contains("100"));
        // Ensure no secret material appears (trivially true for amount errors,
        // but this pattern should be checked for every new error variant).
        assert!(!display.contains("key"));
        assert!(!display.contains("pin"));
        assert!(!display.contains("secret"));
    }

    #[test]
    fn transition_error_contains_state_and_event() {
        let e = TransitionError::IllegalTransition {
            current_state: "CREATED".into(),
            event: "BACKEND_RECONCILED".into(),
        };
        let display = e.to_string();
        assert!(display.contains("CREATED"));
        assert!(display.contains("BACKEND_RECONCILED"));
    }

    #[test]
    fn core_error_from_validation() {
        let v = ValidationError::TransactionExpired;
        let c: CoreError = v.into();
        assert!(matches!(c, CoreError::Validation(_)));
    }

    #[test]
    fn core_error_from_transition() {
        let t = TransitionError::TerminalState {
            current_state: "RECONCILED".into(),
        };
        let c: CoreError = t.into();
        assert!(matches!(c, CoreError::Transition(_)));
    }
}
