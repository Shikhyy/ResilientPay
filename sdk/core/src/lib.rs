//! ResilientPay SDK Core
//!
//! This crate contains the pure domain model for the ResilientPay research prototype.
//! It is intentionally free of Android, network, UI, and transport dependencies so that
//! every domain invariant can be verified in isolation.
//!
//! # Architecture
//!
//! ```text
//! money    — monetary value type (integer minor units, no floating point)
//! types    — domain newtypes (TransactionId, CredentialId, MerchantId, KeyId)
//! envelope — PaymentEnvelopeCore (the signed payment record)
//! credential — OfflineCredential and lifecycle state
//! state_machine — TransactionState, TransactionEvent, apply_event()
//! validation — ValidationResult, field validators, rejection taxonomy
//! errors   — typed error hierarchy
//! ```
//!
//! # Security notes
//!
//! - No floating-point arithmetic is used for monetary values.
//! - No cryptographic primitives are implemented here (see ADR-002, CRYPTO_SPEC.md).
//! - No secrets are stored in these types.
//! - Validation always fails closed: an unrecognised protocol version returns an error.
//!
//! # Protocol reference
//!
//! - `docs/05-protocol/PAYMENT_PROTOCOL.md`
//! - `docs/05-protocol/TRANSACTION_STATE_MACHINE.md`
//! - `docs/04-security/CRYPTO_SPEC.md`
//! - `docs/05-protocol/OFFLINE_CREDENTIAL_SPEC.md`
//! - `docs/06-development/MONEY.md`

pub mod credential;
pub mod envelope;
pub mod errors;
pub mod money;
pub mod state_machine;
pub mod types;
pub mod validation;

// Re-export the most commonly used types at the crate root for ergonomics,
// without flattening the module hierarchy.
pub use credential::{CredentialLifecycleState, OfflineCredential};
pub use envelope::PaymentEnvelopeCore;
pub use errors::{CoreError, TransitionError, ValidationError};
pub use money::Money;
pub use state_machine::{TransactionEvent, TransactionState};
pub use types::{CredentialId, IssuerId, KeyId, MerchantId, MessageId, TransactionId};
pub use validation::{ValidationResult, Validator};

/// The protocol version this crate implements.
///
/// This constant is a signed field in every `PaymentEnvelopeCore`. Changing this
/// value requires a protocol change-control review (see `.agent/CHANGE_CONTROL.md`).
pub const PROTOCOL_VERSION: u32 = 1;
