//! Hash-linked local event ledger.
//!
//! This module provides an append-only local event ledger with SHA-256
//! hash chaining. Each event records the hash of the previous event,
//! creating a tamper-evident sequence.
//!
//! # Protocol reference
//!
//! - ADR-009: Hash-Linked Local Ledger (accepted for research)
//! - `docs/04-security/CRYPTO_SPEC.md` §6 (Hash chaining)
//! - `docs/06-development/DATA_MODEL.md` §1 (TransactionEvent entity)
//!
//! # Chain construction (per CRYPTO_SPEC.md §6)
//!
//! ```text
//! H0 = SHA-256(b"resilientpay:ledger:genesis:v1")
//! Hn = SHA-256(DOMAIN_EVENT || payload_hash || H(n-1))
//! ```
//!
//! where:
//! - `DOMAIN_EVENT = b"resilientpay:ledger:event:v1:"`
//! - `payload_hash = SHA-256(DOMAIN_PAYLOAD || canonical_event_bytes)`
//! - `canonical_event_bytes` is a deterministic encoding of the event fields
//!
//! # Important limitations (ADR-009)
//!
//! - A local hash chain alone **cannot prevent** a fully-compromised device from
//!   rewriting history (attacker controls all data). External checkpoints,
//!   secure hardware, or backend anchoring are needed for stronger guarantees.
//! - Hash chaining does not prevent double-spend — it provides tamper evidence
//!   for the local event sequence only.
//! - These limitations must be reflected in research claims.
//!
//! # Implementation note
//!
//! `InMemoryLedger` is provided for testing and the simulator. Production
//! persistence requires a platform-specific durable store (Room/SQLite on Android).
//! The `LocalLedger` trait is the interface that persistence adapters implement.

use crate::{
    state_machine::{TransactionEvent, TransactionState},
    types::TransactionId,
};
use sha2::{Digest, Sha256};

// ---------------------------------------------------------------------------
// Domain-separation constants
// ---------------------------------------------------------------------------

/// Domain separator for the genesis hash (H0).
const DOMAIN_GENESIS: &[u8] = b"resilientpay:ledger:genesis:v1";

/// Domain separator prepended to each event's chain hash computation.
const DOMAIN_EVENT: &[u8] = b"resilientpay:ledger:event:v1:";

/// Domain separator for the event payload hash.
const DOMAIN_PAYLOAD: &[u8] = b"resilientpay:ledger:payload:v1:";

/// Length of a SHA-256 digest.
pub const DIGEST_LEN: usize = 32;

// ---------------------------------------------------------------------------
// Genesis hash (H0)
// ---------------------------------------------------------------------------

/// Compute H0 — the genesis chain hash.
///
/// This is a fixed, domain-separated value derived from the constant string.
/// It is not secret; its purpose is to anchor the chain at a known value
/// distinct from a valid event hash, preventing trivial hash-extension attacks.
pub fn genesis_chain_hash() -> [u8; DIGEST_LEN] {
    Sha256::digest(DOMAIN_GENESIS).into()
}

// ---------------------------------------------------------------------------
// Ledger event
// ---------------------------------------------------------------------------

/// A single entry in the hash-linked local ledger.
///
/// Each event captures a state transition, a reference to the transaction,
/// the resulting state, the previous chain hash, and the chain hash of this
/// event. The chain hash links events in a tamper-evident sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerEvent {
    /// Monotonically increasing sequence number within this ledger.
    pub sequence: u64,

    /// The transaction this event belongs to.
    pub tx_id: TransactionId,

    /// The event that caused this state transition.
    pub event: TransactionEvent,

    /// The state of the transaction after applying `event`.
    pub resulting_state: TransactionState,

    /// Unix timestamp (seconds) when this event was recorded.
    pub recorded_at_unix_secs: i64,

    /// SHA-256 of the canonical representation of this event's payload.
    /// Computed as: SHA-256(DOMAIN_PAYLOAD || payload_bytes)
    pub payload_hash: [u8; DIGEST_LEN],

    /// The chain hash of the immediately preceding event (or H0 for the first event).
    pub previous_chain_hash: [u8; DIGEST_LEN],

    /// The chain hash of this event.
    /// Computed as: SHA-256(DOMAIN_EVENT || payload_hash || previous_chain_hash)
    pub chain_hash: [u8; DIGEST_LEN],
}

/// Input to `LocalLedger::append`.
///
/// Separates the caller-provided data from the ledger-computed hashes,
/// preventing callers from injecting pre-computed hashes.
#[derive(Debug, Clone)]
pub struct LedgerEventInput {
    pub tx_id: TransactionId,
    pub event: TransactionEvent,
    pub resulting_state: TransactionState,
    pub recorded_at_unix_secs: i64,
    /// Optional additional payload bytes to include in the payload hash.
    /// For example: canonical CBOR bytes of the payment envelope.
    pub payload_bytes: Option<Vec<u8>>,
}

// ---------------------------------------------------------------------------
// Hash computation
// ---------------------------------------------------------------------------

/// Compute the payload hash for an event.
///
/// `SHA-256(DOMAIN_PAYLOAD || tx_id_bytes || event_label || state_label || payload_bytes)`
fn compute_payload_hash(input: &LedgerEventInput) -> [u8; DIGEST_LEN] {
    let mut h = Sha256::new();
    h.update(DOMAIN_PAYLOAD);
    h.update(input.tx_id.as_uuid().as_bytes());
    h.update(input.event.as_str().as_bytes());
    h.update(input.resulting_state.as_str().as_bytes());
    h.update(input.recorded_at_unix_secs.to_le_bytes());
    if let Some(ref payload) = input.payload_bytes {
        h.update(payload);
    }
    h.finalize().into()
}

/// Compute the chain hash for an event.
///
/// `SHA-256(DOMAIN_EVENT || payload_hash || previous_chain_hash)`
fn compute_chain_hash(
    payload_hash: &[u8; DIGEST_LEN],
    previous_chain_hash: &[u8; DIGEST_LEN],
) -> [u8; DIGEST_LEN] {
    let mut h = Sha256::new();
    h.update(DOMAIN_EVENT);
    h.update(payload_hash);
    h.update(previous_chain_hash);
    h.finalize().into()
}

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// Trait for an append-only local event ledger.
///
/// Implementors provide durable event storage. The trait guarantees the
/// hash-chain invariant: each appended event's `previous_chain_hash` must
/// equal the `chain_hash` of the preceding event (or H0 for the first event).
pub trait LocalLedger {
    /// Append a new event to the ledger.
    ///
    /// The ledger implementation computes `payload_hash` and `chain_hash`
    /// from the input. Callers cannot inject pre-computed hashes.
    ///
    /// # Errors
    ///
    /// Returns `LedgerError` if the event cannot be appended (e.g., persistence
    /// failure, or duplicate sequence number in a corrupted store).
    fn append(&mut self, input: LedgerEventInput) -> Result<LedgerEvent, LedgerError>;

    /// Return the chain hash of the most recently appended event, or `None` if
    /// the ledger is empty. Used by callers that need to include the current
    /// chain tip in a new envelope (`previous_event_hash` field).
    fn last_chain_hash(&self) -> Option<&[u8; DIGEST_LEN]>;

    /// Return the total number of events in the ledger.
    fn len(&self) -> usize;

    /// Return `true` if the ledger contains no events.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Verify the integrity of the entire chain.
    ///
    /// Recomputes each event's chain hash and checks that:
    /// 1. The first event's `previous_chain_hash` equals `genesis_chain_hash()`.
    /// 2. Each subsequent event's `previous_chain_hash` equals the preceding
    ///    event's `chain_hash`.
    /// 3. All `chain_hash` values are correctly computed.
    ///
    /// # Errors
    ///
    /// Returns `LedgerError::ChainIntegrityViolation` if any link is broken.
    fn verify_chain(&self) -> Result<(), LedgerError>;
}

// ---------------------------------------------------------------------------
// In-memory ledger (for testing and simulation)
// ---------------------------------------------------------------------------

/// An in-memory, non-persistent ledger for testing and the simulator.
///
/// Events are stored in a `Vec` in append order. This implementation is not
/// suitable for production use (events are lost on process termination) but
/// provides a correct reference implementation of the chain invariant.
#[derive(Debug, Default)]
pub struct InMemoryLedger {
    events: Vec<LedgerEvent>,
}

impl InMemoryLedger {
    /// Create an empty ledger.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Return an iterator over all events in append order.
    pub fn events(&self) -> impl Iterator<Item = &LedgerEvent> {
        self.events.iter()
    }

    /// Return all events for a specific transaction ID, in append order.
    pub fn events_for_tx(&self, tx_id: TransactionId) -> impl Iterator<Item = &LedgerEvent> {
        self.events.iter().filter(move |e| e.tx_id == tx_id)
    }
}

impl LocalLedger for InMemoryLedger {
    fn append(&mut self, input: LedgerEventInput) -> Result<LedgerEvent, LedgerError> {
        let sequence = self.events.len() as u64;

        // The previous chain hash is H0 for the first event, or the last event's hash.
        let previous_chain_hash = self
            .events
            .last()
            .map(|e| e.chain_hash)
            .unwrap_or_else(genesis_chain_hash);

        let payload_hash = compute_payload_hash(&input);
        let chain_hash = compute_chain_hash(&payload_hash, &previous_chain_hash);

        let event = LedgerEvent {
            sequence,
            tx_id: input.tx_id,
            event: input.event,
            resulting_state: input.resulting_state,
            recorded_at_unix_secs: input.recorded_at_unix_secs,
            payload_hash,
            previous_chain_hash,
            chain_hash,
        };

        self.events.push(event.clone());
        Ok(event)
    }

    fn last_chain_hash(&self) -> Option<&[u8; DIGEST_LEN]> {
        self.events.last().map(|e| &e.chain_hash)
    }

    fn len(&self) -> usize {
        self.events.len()
    }

    fn verify_chain(&self) -> Result<(), LedgerError> {
        if self.events.is_empty() {
            return Ok(());
        }

        let genesis = genesis_chain_hash();

        // The first event must point back to the genesis hash.
        if self.events[0].previous_chain_hash != genesis {
            return Err(LedgerError::ChainIntegrityViolation {
                sequence: 0,
                reason: "first event previous_chain_hash does not equal genesis hash".into(),
            });
        }

        for (i, event) in self.events.iter().enumerate() {
            // Recompute chain_hash and compare.
            let recomputed_chain_hash =
                compute_chain_hash(&event.payload_hash, &event.previous_chain_hash);

            if recomputed_chain_hash != event.chain_hash {
                return Err(LedgerError::ChainIntegrityViolation {
                    sequence: event.sequence,
                    reason: "chain_hash recomputation mismatch — event may have been tampered"
                        .into(),
                });
            }

            // For events after the first, the previous hash must equal the preceding event's hash.
            if i > 0 && event.previous_chain_hash != self.events[i - 1].chain_hash {
                return Err(LedgerError::ChainIntegrityViolation {
                    sequence: event.sequence,
                    reason: "previous_chain_hash does not match preceding event chain_hash".into(),
                });
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Ledger errors
// ---------------------------------------------------------------------------

/// Errors produced by ledger operations.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum LedgerError {
    /// The hash chain has been violated — events may have been tampered with.
    #[error("chain integrity violation at sequence {sequence}: {reason}")]
    ChainIntegrityViolation { sequence: u64, reason: String },

    /// A persistence operation failed (returned by production implementations).
    #[error("persistence failure: {reason}")]
    PersistenceFailure { reason: String },

    /// An event with the same sequence number was already recorded.
    #[error("duplicate sequence number {sequence}")]
    DuplicateSequence { sequence: u64 },
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        state_machine::{TransactionEvent, TransactionState},
        types::TransactionId,
    };

    fn make_input(
        tx_id: TransactionId,
        event: TransactionEvent,
        state: TransactionState,
    ) -> LedgerEventInput {
        LedgerEventInput {
            tx_id,
            event,
            resulting_state: state,
            recorded_at_unix_secs: 1_700_000_000,
            payload_bytes: None,
        }
    }

    // -----------------------------------------------------------------------
    // Basic append and chain integrity
    // -----------------------------------------------------------------------

    #[test]
    fn empty_ledger_verifies_clean() {
        let ledger = InMemoryLedger::new();
        assert!(ledger.is_empty());
        assert!(ledger.verify_chain().is_ok());
    }

    #[test]
    fn single_event_appended_and_verified() {
        let mut ledger = InMemoryLedger::new();
        let tx = TransactionId::generate();
        let ev = ledger
            .append(make_input(
                tx,
                TransactionEvent::ValidationRequested,
                TransactionState::Validating,
            ))
            .unwrap();

        assert_eq!(ev.sequence, 0);
        assert_eq!(ev.tx_id, tx);
        assert_eq!(ev.resulting_state, TransactionState::Validating);
        assert_eq!(ev.previous_chain_hash, genesis_chain_hash());
        assert_eq!(ledger.len(), 1);
        assert!(ledger.verify_chain().is_ok());
    }

    #[test]
    fn multiple_events_chain_correctly() {
        let mut ledger = InMemoryLedger::new();
        let tx = TransactionId::generate();

        let sequence = [
            (
                TransactionEvent::ValidationRequested,
                TransactionState::Validating,
            ),
            (
                TransactionEvent::PolicyAccepted,
                TransactionState::Authorized,
            ),
            (TransactionEvent::SignatureCreated, TransactionState::Signed),
        ];

        for (i, (event, state)) in sequence.iter().enumerate() {
            let ev = ledger.append(make_input(tx, *event, *state)).unwrap();
            assert_eq!(ev.sequence, i as u64);
        }

        assert_eq!(ledger.len(), 3);
        assert!(ledger.verify_chain().is_ok());

        // Chain links must be correct.
        let events: Vec<_> = ledger.events().collect();
        assert_eq!(events[1].previous_chain_hash, events[0].chain_hash);
        assert_eq!(events[2].previous_chain_hash, events[1].chain_hash);
    }

    #[test]
    fn last_chain_hash_after_append() {
        let mut ledger = InMemoryLedger::new();
        let tx = TransactionId::generate();
        let ev = ledger
            .append(make_input(
                tx,
                TransactionEvent::ValidationRequested,
                TransactionState::Validating,
            ))
            .unwrap();
        assert_eq!(ledger.last_chain_hash(), Some(&ev.chain_hash));
    }

    // -----------------------------------------------------------------------
    // Hash determinism
    // -----------------------------------------------------------------------

    #[test]
    fn same_input_produces_same_hashes() {
        let tx = TransactionId::generate();
        let input1 = LedgerEventInput {
            tx_id: tx,
            event: TransactionEvent::PolicyAccepted,
            resulting_state: TransactionState::Authorized,
            recorded_at_unix_secs: 1_700_000_000,
            payload_bytes: None,
        };
        let input2 = input1.clone();

        let h1 = compute_payload_hash(&input1);
        let h2 = compute_payload_hash(&input2);
        assert_eq!(h1, h2);
    }

    #[test]
    fn different_events_produce_different_hashes() {
        let tx = TransactionId::generate();
        let i1 = LedgerEventInput {
            tx_id: tx,
            event: TransactionEvent::PolicyAccepted,
            resulting_state: TransactionState::Authorized,
            recorded_at_unix_secs: 1_700_000_000,
            payload_bytes: None,
        };
        let i2 = LedgerEventInput {
            tx_id: tx,
            event: TransactionEvent::SignatureCreated,
            resulting_state: TransactionState::Signed,
            recorded_at_unix_secs: 1_700_000_000,
            payload_bytes: None,
        };
        assert_ne!(compute_payload_hash(&i1), compute_payload_hash(&i2));
    }

    #[test]
    fn payload_bytes_affect_hash() {
        let tx = TransactionId::generate();
        let make_input = |payload: &[u8]| LedgerEventInput {
            tx_id: tx,
            event: TransactionEvent::PolicyAccepted,
            resulting_state: TransactionState::Authorized,
            recorded_at_unix_secs: 1_000_000,
            payload_bytes: Some(payload.to_vec()),
        };
        let h1 = compute_payload_hash(&make_input(b"payload-a"));
        let h2 = compute_payload_hash(&make_input(b"payload-b"));
        assert_ne!(h1, h2);
    }

    // -----------------------------------------------------------------------
    // Tamper detection
    // -----------------------------------------------------------------------

    #[test]
    fn tampered_chain_hash_detected() {
        let mut ledger = InMemoryLedger::new();
        let tx = TransactionId::generate();

        ledger
            .append(make_input(
                tx,
                TransactionEvent::ValidationRequested,
                TransactionState::Validating,
            ))
            .unwrap();
        ledger
            .append(make_input(
                tx,
                TransactionEvent::PolicyAccepted,
                TransactionState::Authorized,
            ))
            .unwrap();

        // Simulate tampering: corrupt the chain_hash of the first event.
        ledger.events[0].chain_hash = [0xFF; DIGEST_LEN];

        let err = ledger.verify_chain().unwrap_err();
        assert!(matches!(
            err,
            LedgerError::ChainIntegrityViolation { sequence: 0, .. }
        ));
    }

    #[test]
    fn tampered_payload_hash_detected() {
        let mut ledger = InMemoryLedger::new();
        let tx = TransactionId::generate();
        ledger
            .append(make_input(
                tx,
                TransactionEvent::ValidationRequested,
                TransactionState::Validating,
            ))
            .unwrap();

        // Tamper with the payload hash — verify_chain should catch it.
        ledger.events[0].payload_hash = [0xAA; DIGEST_LEN];

        let err = ledger.verify_chain().unwrap_err();
        assert!(matches!(
            err,
            LedgerError::ChainIntegrityViolation { sequence: 0, .. }
        ));
    }

    #[test]
    fn tampered_previous_hash_in_middle_detected() {
        let mut ledger = InMemoryLedger::new();
        let tx = TransactionId::generate();
        for (event, state) in [
            (
                TransactionEvent::ValidationRequested,
                TransactionState::Validating,
            ),
            (
                TransactionEvent::PolicyAccepted,
                TransactionState::Authorized,
            ),
            (TransactionEvent::SignatureCreated, TransactionState::Signed),
        ] {
            ledger.append(make_input(tx, event, state)).unwrap();
        }

        // Break the link at event 1 → points to wrong previous hash
        ledger.events[1].previous_chain_hash = [0x00; DIGEST_LEN];

        let err = ledger.verify_chain().unwrap_err();
        assert!(matches!(err, LedgerError::ChainIntegrityViolation { .. }));
    }

    // -----------------------------------------------------------------------
    // events_for_tx filter
    // -----------------------------------------------------------------------

    #[test]
    fn events_for_tx_filters_correctly() {
        let mut ledger = InMemoryLedger::new();
        let tx1 = TransactionId::generate();
        let tx2 = TransactionId::generate();

        ledger
            .append(make_input(
                tx1,
                TransactionEvent::ValidationRequested,
                TransactionState::Validating,
            ))
            .unwrap();
        ledger
            .append(make_input(
                tx2,
                TransactionEvent::ValidationRequested,
                TransactionState::Validating,
            ))
            .unwrap();
        ledger
            .append(make_input(
                tx1,
                TransactionEvent::PolicyAccepted,
                TransactionState::Authorized,
            ))
            .unwrap();

        let tx1_events: Vec<_> = ledger.events_for_tx(tx1).collect();
        assert_eq!(tx1_events.len(), 2);
        assert!(tx1_events.iter().all(|e| e.tx_id == tx1));
    }

    // -----------------------------------------------------------------------
    // Genesis hash
    // -----------------------------------------------------------------------

    #[test]
    fn genesis_hash_is_deterministic() {
        assert_eq!(genesis_chain_hash(), genesis_chain_hash());
    }

    #[test]
    fn genesis_hash_is_not_all_zeros() {
        assert_ne!(genesis_chain_hash(), [0u8; DIGEST_LEN]);
    }

    #[test]
    fn first_event_previous_hash_is_genesis() {
        let mut ledger = InMemoryLedger::new();
        let ev = ledger
            .append(make_input(
                TransactionId::generate(),
                TransactionEvent::ValidationRequested,
                TransactionState::Validating,
            ))
            .unwrap();
        assert_eq!(ev.previous_chain_hash, genesis_chain_hash());
    }
}
