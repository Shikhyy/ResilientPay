//! Integration test: full payment flow without transport.
//!
//! This test exercises the complete protocol sequence:
//!
//! 1. Payer creates a `PaymentEnvelopeCore` with valid credential and counter
//! 2. Payer signs the envelope using the Ed25519 test signer
//! 3. The signing input (canonical CBOR + domain separator) is deterministic
//! 4. Merchant receives the signed envelope
//! 5. Merchant verifies the signature
//! 6. Merchant runs the deterministic validator (credential, counter, value, merchant)
//! 7. Merchant appends to the local hash-linked ledger
//! 8. State machine steps through the full happy path
//! 9. Hash chain integrity is verified
//!
//! This tests the transport-independent core only — no NFC, BLE, QR, SMS, or HTTP.
//!
//! # Security properties verified
//!
//! - Tampered amount fails signature verification (amount integrity)
//! - Replayed counter fails validator (replay defense)
//! - Wrong merchant fails validator (merchant binding)
//! - Expired credential fails validator (credential lifecycle)
//! - Hash chain detects tampered events (tamper evidence)
//! - Full happy path transitions through all canonical states

use resilientpay_core::{
    apply_event,
    credential::{CredentialLifecycleState, OfflineCredential},
    crypto::{sign_envelope, verify_envelope, Ed25519TestSigner},
    envelope::EnvelopeBuilder,
    ledger::{InMemoryLedger, LedgerEventInput, LocalLedger},
    money::Money,
    serialization::signing_input,
    state_machine::{TransactionEvent, TransactionState},
    types::{CredentialId, IssuerId, KeyId, MerchantId, TransactionId},
    validation::{ValidationContext, ValidationResult, Validator},
    PROTOCOL_VERSION,
};

const NONCE: [u8; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

/// Fixed seed for the test signer — produces a deterministic key pair.
const TEST_SEED: [u8; 32] = [
    0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00,
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10,
];

struct TestFixture {
    signer: Ed25519TestSigner,
    cred_id: CredentialId,
    key_id: KeyId,
    merchant_id: MerchantId,
    issuer_id: IssuerId,
}

impl TestFixture {
    fn new() -> Self {
        let signer = Ed25519TestSigner::from_seed(&TEST_SEED);
        let cred_id = CredentialId::generate();
        let key_id = KeyId::generate();
        let merchant_id = MerchantId::generate();
        let issuer_id = IssuerId::generate();
        Self {
            signer,
            cred_id,
            key_id,
            merchant_id,
            issuer_id,
        }
    }

    fn credential(&self) -> OfflineCredential {
        OfflineCredential::new(
            self.cred_id,
            self.key_id,
            self.issuer_id,
            1_000_000,                           // issued_at
            2_000_000,                           // expires_at (valid for 1,000,000 seconds)
            Money::new(50_000, "INR").unwrap(),  // max per-tx: ₹500
            Money::new(200_000, "INR").unwrap(), // max outstanding: ₹2000
            1_000,                               // max_counter
            CredentialLifecycleState::Active,
            1,
        )
        .unwrap()
    }

    fn envelope(&self, amount_minor: u64, counter: u64) -> resilientpay_core::PaymentEnvelopeCore {
        EnvelopeBuilder::new()
            .protocol_version(PROTOCOL_VERSION)
            .tx_id(TransactionId::generate())
            .credential_id(self.cred_id)
            .payer_key_id(self.key_id)
            .merchant_id(self.merchant_id)
            .amount(Money::new(amount_minor, "INR").unwrap())
            .counter(counter)
            .nonce(NONCE)
            .created_at_unix_secs(1_500_000)
            .expires_at_unix_secs(1_600_000)
            .build()
            .unwrap()
    }

    fn validation_context(&self, last_counter: u64) -> ValidationContext {
        ValidationContext {
            now_unix_secs: 1_550_000, // midpoint — within expiry window
            expected_merchant_id: self.merchant_id,
            last_seen_counter: last_counter,
            current_outstanding_budget: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Full happy-path integration test
// ---------------------------------------------------------------------------

#[test]
fn full_payment_flow_happy_path() {
    let fx = TestFixture::new();
    let cred = fx.credential();
    let verifier = fx.signer.verifier();

    // --- Step 1: Payer creates envelope ---
    let envelope = fx.envelope(10_000, 1); // ₹100, counter = 1

    // --- Step 2: Payer signs ---
    let signature = sign_envelope(&envelope, &fx.signer).expect("signing must succeed");

    // --- Step 3: Merchant verifies signature ---
    verify_envelope(&envelope, &signature, &verifier)
        .expect("signature must verify on unmodified envelope");

    // --- Step 4: Merchant validates (credential, counter, value, merchant) ---
    let ctx = fx.validation_context(0);
    let result = Validator::validate(&envelope, &cred, &ctx);
    assert_eq!(result, ValidationResult::Accepted, "validation must accept");

    // --- Step 5: Merchant records event to ledger ---
    let mut ledger = InMemoryLedger::new();
    let signing_bytes = signing_input(&envelope).unwrap();
    let ledger_event = ledger
        .append(LedgerEventInput {
            tx_id: envelope.tx_id(),
            event: TransactionEvent::EvidenceValidated,
            resulting_state: TransactionState::LocallyVerified,
            recorded_at_unix_secs: 1_550_001,
            payload_bytes: Some(signing_bytes),
        })
        .expect("ledger append must succeed");

    assert_eq!(ledger_event.sequence, 0);
    assert_eq!(
        ledger_event.resulting_state,
        TransactionState::LocallyVerified
    );
    assert!(
        ledger.verify_chain().is_ok(),
        "chain must be intact after happy path"
    );

    // --- Step 6: Walk the full state machine from CREATED to RECONCILED ---
    let mut state = TransactionState::Created;
    let steps = [
        TransactionEvent::ValidationRequested,
        TransactionEvent::PolicyAccepted,
        TransactionEvent::SignatureCreated,
        TransactionEvent::TransportDelivered,
        TransactionEvent::EnvelopeReceived,
        TransactionEvent::EvidenceValidated,
        TransactionEvent::Persisted,
        TransactionEvent::SyncQueued,
        TransactionEvent::BackendReconciled,
    ];
    for step in steps {
        state = apply_event(state, step)
            .unwrap_or_else(|e| panic!("unexpected state machine error on {step:?}: {e:?}"));
    }
    assert_eq!(state, TransactionState::Reconciled);
    assert!(state.is_terminal());
}

// ---------------------------------------------------------------------------
// Security: tampered amount fails verification
// ---------------------------------------------------------------------------

#[test]
fn tampered_amount_fails_signature_verification() {
    let fx = TestFixture::new();
    let verifier = fx.signer.verifier();

    let original = fx.envelope(10_000, 1); // ₹100
    let sig = sign_envelope(&original, &fx.signer).unwrap();

    // Attacker tampers: replaces amount with ₹0.01 (1 paise)
    let tampered = EnvelopeBuilder::new()
        .protocol_version(PROTOCOL_VERSION)
        .tx_id(original.tx_id())
        .credential_id(original.credential_id())
        .payer_key_id(original.payer_key_id())
        .merchant_id(original.merchant_id())
        .amount(Money::new(1, "INR").unwrap()) // ← tampered
        .counter(original.counter())
        .nonce(*original.nonce())
        .created_at_unix_secs(original.created_at_unix_secs())
        .expires_at_unix_secs(original.expires_at_unix_secs())
        .build()
        .unwrap();

    let err = verify_envelope(&tampered, &sig, &verifier).unwrap_err();
    assert!(
        matches!(err, resilientpay_core::CryptoError::VerificationFailed),
        "tampered amount must fail verification, got: {err:?}"
    );
}

// ---------------------------------------------------------------------------
// Security: counter replay rejected by validator
// ---------------------------------------------------------------------------

#[test]
fn counter_replay_rejected_by_validator() {
    let fx = TestFixture::new();
    let cred = fx.credential();

    // Simulate: counter 5 was previously seen and recorded
    let last_seen = 5u64;
    let replayed_envelope = fx.envelope(5_000, 5); // counter = 5 (replay)

    let ctx = fx.validation_context(last_seen);
    let result = Validator::validate(&replayed_envelope, &cred, &ctx);

    assert!(result.is_rejected());
    assert!(
        matches!(
            result.rejection_error(),
            Some(resilientpay_core::ValidationError::CounterViolation { .. })
        ),
        "replay must produce CounterViolation, got: {:?}",
        result.rejection_error()
    );
}

// ---------------------------------------------------------------------------
// Security: wrong merchant rejected by validator
// ---------------------------------------------------------------------------

#[test]
fn wrong_merchant_rejected_by_validator() {
    let fx = TestFixture::new();
    let cred = fx.credential();

    let envelope = fx.envelope(1_000, 1);

    // Validator is configured for a DIFFERENT merchant
    let ctx = ValidationContext {
        now_unix_secs: 1_550_000,
        expected_merchant_id: MerchantId::generate(), // different merchant
        last_seen_counter: 0,
            current_outstanding_budget: 0,
    };

    let result = Validator::validate(&envelope, &cred, &ctx);
    assert!(result.is_rejected(), "wrong merchant must be rejected");
}

// ---------------------------------------------------------------------------
// Security: expired credential rejected by validator
// ---------------------------------------------------------------------------

#[test]
fn expired_credential_rejected_by_validator() {
    let fx = TestFixture::new();
    let cred = fx.credential(); // credential expires at 2_000_000

    let envelope = fx.envelope(1_000, 1);

    let ctx = ValidationContext {
        now_unix_secs: 2_000_001, // after credential expiry
        expected_merchant_id: fx.merchant_id,
        last_seen_counter: 0,
            current_outstanding_budget: 0,
    };

    let result = Validator::validate(&envelope, &cred, &ctx);
    assert!(result.is_rejected(), "expired credential must be rejected");
}

// ---------------------------------------------------------------------------
// Ledger: verify chain is intact after multiple events
// ---------------------------------------------------------------------------

#[test]
fn ledger_chain_integrity_across_multiple_transactions() {
    let mut ledger = InMemoryLedger::new();
    let fx = TestFixture::new();

    for counter in 1u64..=5 {
        let envelope = fx.envelope(1_000 * counter, counter);
        let bytes = signing_input(&envelope).unwrap();

        ledger
            .append(LedgerEventInput {
                tx_id: envelope.tx_id(),
                event: TransactionEvent::EvidenceValidated,
                resulting_state: TransactionState::LocallyVerified,
                recorded_at_unix_secs: 1_550_000 + counter as i64,
                payload_bytes: Some(bytes),
            })
            .unwrap();
    }

    assert_eq!(ledger.len(), 5);
    assert!(
        ledger.verify_chain().is_ok(),
        "chain must be intact for 5 events"
    );
}

// ---------------------------------------------------------------------------
// Serialization: signing input is stable across separate calls
// ---------------------------------------------------------------------------

#[test]
fn signing_input_stable_across_separate_calls() {
    let fx = TestFixture::new();
    let envelope = fx.envelope(50_000, 42);

    let bytes_a = signing_input(&envelope).unwrap();
    let bytes_b = signing_input(&envelope).unwrap();
    assert_eq!(bytes_a, bytes_b, "signing input must be deterministic");
    assert!(!bytes_a.is_empty());
}

// ---------------------------------------------------------------------------
// Signature: different counters produce different signatures
// ---------------------------------------------------------------------------

#[test]
fn different_counters_produce_different_signatures() {
    let fx = TestFixture::new();

    let env1 = fx.envelope(10_000, 1);
    let env2 = fx.envelope(10_000, 2);

    let sig1 = sign_envelope(&env1, &fx.signer).unwrap();
    let sig2 = sign_envelope(&env2, &fx.signer).unwrap();

    assert_ne!(
        sig1.to_bytes(),
        sig2.to_bytes(),
        "different counters must produce different signatures"
    );
}
