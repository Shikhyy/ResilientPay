//! Transaction state machine.
//!
//! Implements the canonical state machine defined in:
//! `docs/05-protocol/TRANSACTION_STATE_MACHINE.md`
//!
//! # Canonical states
//!
//! ```text
//! CREATED
//!   ↓
//! VALIDATING
//!   ↓
//! AUTHORIZED
//!   ↓
//! SIGNED
//!   ↓
//! TRANSFERRED
//!   ↓
//! RECEIVED
//!   ↓
//! LOCALLY_VERIFIED
//!   ↓
//! LOCALLY_RECORDED
//!   ↓
//! SYNC_PENDING
//!   ├────────→ RECONCILED  (terminal)
//!   ├────────→ CONFLICT    (terminal)
//!   └────────→ REJECTED    (terminal)
//! ```
//!
//! # Implementation rules
//!
//! - Transitions are encoded in a single exhaustive match. No booleans.
//! - Every invalid transition returns `TransitionError::IllegalTransition`.
//! - Terminal states return `TransitionError::TerminalState` for any event.
//! - A transaction must not simultaneously claim mutually incompatible states.
//!
//! # Testing rule
//!
//! Every valid transition AND every documented invalid transition must be covered.

use crate::errors::TransitionError;

/// Canonical transaction state as specified in the state machine document.
///
/// The naming exactly matches the normative state names in
/// `docs/05-protocol/TRANSACTION_STATE_MACHINE.md`. Do not rename states without
/// a protocol change-control review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransactionState {
    /// Transaction object created; no validation performed yet.
    Created,

    /// Validation checks are in progress (policy, credential status, counter).
    Validating,

    /// All deterministic policy checks passed. The transaction is authorized
    /// for signing but not yet signed.
    Authorized,

    /// A cryptographic signature has been produced and is attached.
    /// The payment envelope is now an authenticated record.
    Signed,

    /// The signed envelope has been handed to a transport adapter.
    /// Transport success does NOT equal payment authorization.
    Transferred,

    /// The merchant side has received and parsed the envelope.
    Received,

    /// The merchant has verified: signature, credential, counter, policy, merchant ID.
    LocallyVerified,

    /// Evidence has been durably persisted to the local ledger.
    LocallyRecorded,

    /// Reconciliation with the backend is pending.
    SyncPending,

    /// Backend accepted the evidence. This is the prototype's `RECONCILED` state.
    /// **Terminal.** No further transitions are permitted.
    ///
    /// Note: this represents prototype reconciliation, not real UPI settlement.
    Reconciled,

    /// The backend settlement engine has completed fund transfer.
    /// **Terminal.** Absolute end of lifecycle.
    Settled,

    /// The backend or a local rule rejected this transaction.
    /// **Terminal.** Requires an explicit new protocol event to remediate (outside
    /// the scope of this state machine).
    Rejected,

    /// Contradictory evidence was detected during reconciliation.
    /// **Terminal.**
    Conflict,
}

impl TransactionState {
    /// Return `true` if no further transitions are permitted from this state.
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Settled | Self::Rejected | Self::Conflict)
    }

    /// Return a human-readable name matching the normative specification label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "CREATED",
            Self::Validating => "VALIDATING",
            Self::Authorized => "AUTHORIZED",
            Self::Signed => "SIGNED",
            Self::Transferred => "TRANSFERRED",
            Self::Received => "RECEIVED",
            Self::LocallyVerified => "LOCALLY_VERIFIED",
            Self::LocallyRecorded => "LOCALLY_RECORDED",
            Self::SyncPending => "SYNC_PENDING",
            Self::Reconciled => "RECONCILED",
            Self::Rejected => "REJECTED",
            Self::Conflict => "CONFLICT",
            Self::Settled => "SETTLED",
        }
    }
}

impl std::fmt::Display for TransactionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Events that drive state transitions.
///
/// The event vocabulary matches the normative reference transition table in
/// `docs/05-protocol/TRANSACTION_STATE_MACHINE.md` (§ Reference transition table).
/// Do not add, remove, or rename events without a protocol change-control review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransactionEvent {
    /// Backend worker completed clearing and settlement.
    BackendSettled,
    /// A validation pass has been requested for this transaction.
    ValidationRequested,

    /// All deterministic policy checks passed.
    PolicyAccepted,

    /// A policy or validation check failed (transitions to REJECTED).
    PolicyRejected,

    /// A cryptographic signature was successfully produced.
    SignatureCreated,

    /// The signed envelope was accepted by a transport adapter for delivery.
    TransportDelivered,

    /// A transport failure occurred (does NOT transition state — caller must
    /// retry or report; transport failure is not payment failure).
    ///
    /// This event is included to make the failure path explicit. It is a no-op
    /// on the state machine: the state remains `Transferred` (or whatever the
    /// current state is). The transport layer must decide whether to retry.
    TransportFailed,

    /// The merchant side parsed the incoming envelope.
    EnvelopeReceived,

    /// The merchant verified the signature, credential, counter, and policy.
    EvidenceValidated,

    /// Verification failed on the merchant side (transitions to REJECTED).
    EvidenceRejected,

    /// Evidence was durably persisted to the local ledger.
    Persisted,

    /// Reconciliation with the backend was queued.
    SyncQueued,

    /// The backend accepted the submitted evidence.
    BackendReconciled,

    /// The backend rejected the submitted evidence.
    BackendRejected,

    /// The backend found conflicting evidence.
    BackendConflict,
}

impl TransactionEvent {
    /// Return a human-readable name for logging and error messages.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ValidationRequested => "VALIDATION_REQUESTED",
            Self::PolicyAccepted => "POLICY_ACCEPTED",
            Self::PolicyRejected => "POLICY_REJECTED",
            Self::SignatureCreated => "SIGNATURE_CREATED",
            Self::TransportDelivered => "TRANSPORT_DELIVERED",
            Self::TransportFailed => "TRANSPORT_FAILED",
            Self::EnvelopeReceived => "ENVELOPE_RECEIVED",
            Self::EvidenceValidated => "EVIDENCE_VALIDATED",
            Self::EvidenceRejected => "EVIDENCE_REJECTED",
            Self::Persisted => "PERSISTED",
            Self::SyncQueued => "SYNC_QUEUED",
            Self::BackendReconciled => "BACKEND_RECONCILED",
            Self::BackendRejected => "BACKEND_REJECTED",
            Self::BackendConflict => "BACKEND_CONFLICT",
            Self::BackendSettled => "BACKEND_SETTLED",
        }
    }
}

impl std::fmt::Display for TransactionEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ---------------------------------------------------------------------------
// Transition function
// ---------------------------------------------------------------------------

/// Apply a `TransactionEvent` to a `TransactionState`, returning the resulting state.
///
/// This is the authoritative transition function. All permitted transitions are
/// listed explicitly. Everything else is rejected.
///
/// # Errors
///
/// - `TransitionError::TerminalState` — the current state is terminal.
/// - `TransitionError::IllegalTransition` — the event is not permitted from the
///   current state by the specification.
///
/// # Design note
///
/// `TransportFailed` is intentionally a no-op — it returns the *same* state
/// unchanged, because transport failure does not alter payment state. The caller
/// is responsible for retry logic, timeout handling, and user notification.
pub fn apply_event(
    current: TransactionState,
    event: TransactionEvent,
) -> Result<TransactionState, TransitionError> {
    // Terminal states never accept events.
    if current.is_terminal() {
        return Err(TransitionError::TerminalState {
            current_state: current.to_string(),
        });
    }

    let next = match (current, event) {
        // --- Payer-side happy path ---
        (TransactionState::Created, TransactionEvent::ValidationRequested) => {
            TransactionState::Validating
        }
        (TransactionState::Validating, TransactionEvent::PolicyAccepted) => {
            TransactionState::Authorized
        }
        (TransactionState::Authorized, TransactionEvent::SignatureCreated) => {
            TransactionState::Signed
        }
        (TransactionState::Signed, TransactionEvent::TransportDelivered) => {
            TransactionState::Transferred
        }

        // --- Rejection paths (non-terminal source states) ---
        (TransactionState::Validating, TransactionEvent::PolicyRejected) => {
            TransactionState::Rejected
        }

        // --- Transport failure: no state change (explicit no-op) ---
        // Transport failure is not payment failure. The envelope is still SIGNED;
        // the transport must retry or the user must choose another transport.
        (TransactionState::Signed, TransactionEvent::TransportFailed) => TransactionState::Signed,
        (TransactionState::Transferred, TransactionEvent::TransportFailed) => {
            TransactionState::Transferred
        }

        // --- Merchant-side happy path ---
        (TransactionState::Transferred, TransactionEvent::EnvelopeReceived) => {
            TransactionState::Received
        }
        (TransactionState::Received, TransactionEvent::EvidenceValidated) => {
            TransactionState::LocallyVerified
        }
        (TransactionState::LocallyVerified, TransactionEvent::Persisted) => {
            TransactionState::LocallyRecorded
        }
        (TransactionState::LocallyRecorded, TransactionEvent::SyncQueued) => {
            TransactionState::SyncPending
        }

        // --- Merchant-side rejection ---
        (TransactionState::Received, TransactionEvent::EvidenceRejected) => {
            TransactionState::Rejected
        }

        // --- Backend reconciliation outcomes (from SYNC_PENDING) ---
        (TransactionState::SyncPending, TransactionEvent::BackendReconciled) => {
            TransactionState::Reconciled
        }
        (TransactionState::SyncPending, TransactionEvent::BackendRejected) => {
            TransactionState::Rejected
        }
        (TransactionState::SyncPending, TransactionEvent::BackendConflict) => {
            TransactionState::Conflict
        }

        // --- Backend settlement (from RECONCILED) ---
        (TransactionState::Reconciled, TransactionEvent::BackendSettled) => {
            TransactionState::Settled
        }

        // --- Everything else is illegal ---
        _ => {
            return Err(TransitionError::IllegalTransition {
                current_state: current.to_string(),
                event: event.to_string(),
            });
        }
    };

    Ok(next)
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Happy-path transitions (payer side)
    // -----------------------------------------------------------------------

    #[test]
    fn created_to_validating() {
        assert_eq!(
            apply_event(
                TransactionState::Created,
                TransactionEvent::ValidationRequested
            )
            .unwrap(),
            TransactionState::Validating
        );
    }

    #[test]
    fn validating_to_authorized() {
        assert_eq!(
            apply_event(
                TransactionState::Validating,
                TransactionEvent::PolicyAccepted
            )
            .unwrap(),
            TransactionState::Authorized
        );
    }

    #[test]
    fn authorized_to_signed() {
        assert_eq!(
            apply_event(
                TransactionState::Authorized,
                TransactionEvent::SignatureCreated
            )
            .unwrap(),
            TransactionState::Signed
        );
    }

    #[test]
    fn signed_to_transferred() {
        assert_eq!(
            apply_event(
                TransactionState::Signed,
                TransactionEvent::TransportDelivered
            )
            .unwrap(),
            TransactionState::Transferred
        );
    }

    // -----------------------------------------------------------------------
    // Happy-path transitions (merchant side)
    // -----------------------------------------------------------------------

    #[test]
    fn transferred_to_received() {
        assert_eq!(
            apply_event(
                TransactionState::Transferred,
                TransactionEvent::EnvelopeReceived
            )
            .unwrap(),
            TransactionState::Received
        );
    }

    #[test]
    fn received_to_locally_verified() {
        assert_eq!(
            apply_event(
                TransactionState::Received,
                TransactionEvent::EvidenceValidated
            )
            .unwrap(),
            TransactionState::LocallyVerified
        );
    }

    #[test]
    fn locally_verified_to_locally_recorded() {
        assert_eq!(
            apply_event(
                TransactionState::LocallyVerified,
                TransactionEvent::Persisted
            )
            .unwrap(),
            TransactionState::LocallyRecorded
        );
    }

    #[test]
    fn locally_recorded_to_sync_pending() {
        assert_eq!(
            apply_event(
                TransactionState::LocallyRecorded,
                TransactionEvent::SyncQueued
            )
            .unwrap(),
            TransactionState::SyncPending
        );
    }

    // -----------------------------------------------------------------------
    // Backend reconciliation outcomes
    // -----------------------------------------------------------------------

    #[test]
    fn sync_pending_to_reconciled() {
        let next = apply_event(
            TransactionState::SyncPending,
            TransactionEvent::BackendReconciled,
        )
        .unwrap();
        assert_eq!(next, TransactionState::Reconciled);
        assert!(!next.is_terminal());

        let settled = apply_event(next, TransactionEvent::BackendSettled).unwrap();
        assert_eq!(settled, TransactionState::Settled);
        assert!(settled.is_terminal());
    }

    #[test]
    fn sync_pending_to_rejected() {
        let next = apply_event(
            TransactionState::SyncPending,
            TransactionEvent::BackendRejected,
        )
        .unwrap();
        assert_eq!(next, TransactionState::Rejected);
        assert!(next.is_terminal());
    }

    #[test]
    fn sync_pending_to_conflict() {
        let next = apply_event(
            TransactionState::SyncPending,
            TransactionEvent::BackendConflict,
        )
        .unwrap();
        assert_eq!(next, TransactionState::Conflict);
        assert!(next.is_terminal());
    }

    // -----------------------------------------------------------------------
    // Rejection paths
    // -----------------------------------------------------------------------

    #[test]
    fn validating_policy_rejected_goes_to_rejected() {
        let next = apply_event(
            TransactionState::Validating,
            TransactionEvent::PolicyRejected,
        )
        .unwrap();
        assert_eq!(next, TransactionState::Rejected);
    }

    #[test]
    fn received_evidence_rejected_goes_to_rejected() {
        let next = apply_event(
            TransactionState::Received,
            TransactionEvent::EvidenceRejected,
        )
        .unwrap();
        assert_eq!(next, TransactionState::Rejected);
    }

    // -----------------------------------------------------------------------
    // Transport failure: explicit no-op
    // -----------------------------------------------------------------------

    #[test]
    fn signed_transport_failed_remains_signed() {
        assert_eq!(
            apply_event(TransactionState::Signed, TransactionEvent::TransportFailed).unwrap(),
            TransactionState::Signed,
            "transport failure must not alter SIGNED state"
        );
    }

    #[test]
    fn transferred_transport_failed_remains_transferred() {
        assert_eq!(
            apply_event(
                TransactionState::Transferred,
                TransactionEvent::TransportFailed
            )
            .unwrap(),
            TransactionState::Transferred
        );
    }

    // -----------------------------------------------------------------------
    // Terminal state rejection
    // -----------------------------------------------------------------------

    #[test]
    fn reconciled_transitions_to_settled() {
        let next = apply_event(
            TransactionState::Reconciled,
            TransactionEvent::BackendSettled,
        )
        .expect("Reconciled + BackendSettled must transition to Settled");
        assert_eq!(next, TransactionState::Settled);
    }

    #[test]
    fn settled_rejects_all_events() {
        for event in [
            TransactionEvent::ValidationRequested,
            TransactionEvent::BackendReconciled,
            TransactionEvent::BackendRejected,
            TransactionEvent::BackendSettled,
            TransactionEvent::SyncQueued,
        ] {
            let err = apply_event(TransactionState::Settled, event).unwrap_err();
            assert!(
                matches!(err, TransitionError::TerminalState { .. }),
                "expected TerminalState error for event {:?} on Settled",
                event
            );
        }
    }

    #[test]
    fn rejected_rejects_all_events() {
        let err = apply_event(
            TransactionState::Rejected,
            TransactionEvent::BackendReconciled,
        )
        .unwrap_err();
        assert!(matches!(err, TransitionError::TerminalState { .. }));
    }

    #[test]
    fn conflict_rejects_all_events() {
        let err = apply_event(
            TransactionState::Conflict,
            TransactionEvent::BackendReconciled,
        )
        .unwrap_err();
        assert!(matches!(err, TransitionError::TerminalState { .. }));
    }

    // -----------------------------------------------------------------------
    // Illegal transitions (specification: these are NOT permitted)
    // -----------------------------------------------------------------------

    #[test]
    fn created_cannot_jump_to_reconciled() {
        let err = apply_event(
            TransactionState::Created,
            TransactionEvent::BackendReconciled,
        )
        .unwrap_err();
        assert!(matches!(err, TransitionError::IllegalTransition { .. }));
    }

    #[test]
    fn authorized_cannot_go_back_to_created() {
        let err = apply_event(
            TransactionState::Authorized,
            TransactionEvent::ValidationRequested,
        )
        .unwrap_err();
        assert!(matches!(err, TransitionError::IllegalTransition { .. }));
    }

    #[test]
    fn signed_cannot_skip_to_reconciled() {
        let err = apply_event(
            TransactionState::Signed,
            TransactionEvent::BackendReconciled,
        )
        .unwrap_err();
        assert!(matches!(err, TransitionError::IllegalTransition { .. }));
    }

    #[test]
    fn transferred_cannot_go_to_reconciled_directly() {
        let err = apply_event(
            TransactionState::Transferred,
            TransactionEvent::BackendReconciled,
        )
        .unwrap_err();
        assert!(matches!(err, TransitionError::IllegalTransition { .. }));
    }

    #[test]
    fn created_cannot_accept_signature_event() {
        let err = apply_event(
            TransactionState::Created,
            TransactionEvent::SignatureCreated,
        )
        .unwrap_err();
        assert!(matches!(err, TransitionError::IllegalTransition { .. }));
    }

    #[test]
    fn sync_pending_cannot_loop_back_to_created() {
        let err = apply_event(
            TransactionState::SyncPending,
            TransactionEvent::ValidationRequested,
        )
        .unwrap_err();
        assert!(matches!(err, TransitionError::IllegalTransition { .. }));
    }

    // -----------------------------------------------------------------------
    // State properties
    // -----------------------------------------------------------------------

    #[test]
    fn only_terminal_states_report_terminal() {
        let terminal = [
            TransactionState::Settled,
            TransactionState::Rejected,
            TransactionState::Conflict,
        ];
        let non_terminal = [
            TransactionState::Created,
            TransactionState::Validating,
            TransactionState::Authorized,
            TransactionState::Signed,
            TransactionState::Transferred,
            TransactionState::Received,
            TransactionState::LocallyVerified,
            TransactionState::LocallyRecorded,
            TransactionState::SyncPending,
            TransactionState::Reconciled,
        ];
        for s in terminal {
            assert!(s.is_terminal(), "{} must be terminal", s);
        }
        for s in non_terminal {
            assert!(!s.is_terminal(), "{} must not be terminal", s);
        }
    }

    #[test]
    fn state_display_matches_spec_names() {
        assert_eq!(TransactionState::Created.to_string(), "CREATED");
        assert_eq!(TransactionState::Reconciled.to_string(), "RECONCILED");
        assert_eq!(
            TransactionState::LocallyVerified.to_string(),
            "LOCALLY_VERIFIED"
        );
        assert_eq!(TransactionState::SyncPending.to_string(), "SYNC_PENDING");
    }

    #[test]
    fn full_happy_path_sequence() {
        // Walk the entire payer→merchant→reconciliation happy path.
        let states = [
            (
                TransactionEvent::ValidationRequested,
                TransactionState::Validating,
            ),
            (
                TransactionEvent::PolicyAccepted,
                TransactionState::Authorized,
            ),
            (TransactionEvent::SignatureCreated, TransactionState::Signed),
            (
                TransactionEvent::TransportDelivered,
                TransactionState::Transferred,
            ),
            (
                TransactionEvent::EnvelopeReceived,
                TransactionState::Received,
            ),
            (
                TransactionEvent::EvidenceValidated,
                TransactionState::LocallyVerified,
            ),
            (
                TransactionEvent::Persisted,
                TransactionState::LocallyRecorded,
            ),
            (TransactionEvent::SyncQueued, TransactionState::SyncPending),
            (
                TransactionEvent::BackendReconciled,
                TransactionState::Reconciled,
            ),
            (
                TransactionEvent::BackendSettled,
                TransactionState::Settled,
            ),
        ];

        let mut current = TransactionState::Created;
        for (event, expected_next) in states {
            let next = apply_event(current, event)
                .unwrap_or_else(|e| panic!("unexpected error on {:?}: {:?}", event, e));
            assert_eq!(next, expected_next, "after {:?}", event);
            current = next;
        }
        assert!(current.is_terminal());
    }
}
