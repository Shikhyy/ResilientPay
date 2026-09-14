//! # ResilientPay Core SDK
//!
//! `resilientpay-core` is the transport-independent Rust domain layer for the
//! ResilientPay research payment system. It provides:
//!
//! - **Domain types**: Money, IDs, payment envelopes, offline credentials
//! - **Transaction state machine**: canonical states and events (extended table)
//! - **Deterministic validation**: ordered policy/credential/counter/expiry checks
//! - **Canonical serialization**: CBOR-encoded signing input (ADR-008)
//! - **Cryptographic boundary**: Ed25519 sign/verify interface (ADR-002, CRYPTO_SPEC.md)
//! - **Hash-linked local ledger**: append-only tamper-evident event log (ADR-009)
//!
//! # Architecture
//!
//! This crate has no Android, network, or filesystem dependencies. Android
//! applications depend on this crate through the Kotlin/JNI wrapper (`sdk/android/`).
//! Transport adapters call into this crate but do not contain payment logic.
//!
//! # Protocol version
//!
//! [`PROTOCOL_VERSION`] identifies the prototype protocol iteration. It is a
//! signed field in every payment envelope. Changing it requires a protocol-level
//! change-control review.

/// Protocol version for this research prototype.
///
/// This value is a signed field in every `PaymentEnvelopeCore`. Any change to
/// this constant requires a protocol change-control review and an ADR update.
///
/// # References
/// - `docs/05-protocol/PAYMENT_PROTOCOL.md` §1 (Protocol versioning)
pub const PROTOCOL_VERSION: u32 = 1;

// ---------------------------------------------------------------------------
// Module declarations
// ---------------------------------------------------------------------------

pub mod credential;
pub mod crypto;
pub mod key_manager;

pub mod envelope;
pub mod errors;
pub mod ledger;
pub mod money;
pub mod serialization;
pub mod state_machine;
pub mod types;
pub mod validation;

// ---------------------------------------------------------------------------
// Convenience re-exports
// ---------------------------------------------------------------------------

pub use credential::{CredentialLifecycleState, OfflineCredential};
pub use crypto::{

    sign_envelope, verify_envelope, CryptoError, Ed25519TestSigner, Ed25519Verifier, PublicKey,
    Signature, Signer, Verifier,
};
pub use envelope::{EnvelopeBuilder, PaymentEnvelopeCore};
pub use errors::{CoreError, TransitionError, ValidationError};
pub use ledger::{
    genesis_chain_hash, InMemoryLedger, LedgerError, LedgerEvent, LedgerEventInput, LocalLedger,
    DIGEST_LEN,
};
pub use money::Money;
pub use serialization::{encode_envelope_cbor, signing_input, SerializationError};
pub use state_machine::{apply_event, TransactionEvent, TransactionState};
pub use types::{CredentialId, IssuerId, KeyId, MerchantId, MessageId, TransactionId};
pub use validation::{ValidationContext, ValidationResult, Validator};
pub use key_manager::KeyManager;
uniffi::setup_scaffolding!();
pub mod ffi;
