//! Envelope validation layer.
//!
//! This module applies the ordered validation sequence defined in:
//! `docs/05-protocol/PAYMENT_PROTOCOL.md` §Processing order
//!
//! # Processing order (normative)
//!
//! 1. Parse/schema validation          — done before this module (by type construction)
//! 2. Protocol-version validation      — checked in `PaymentEnvelopeCore::new`
//! 3. Signature verification           — NOT here (requires crypto; see CRYPTO_SPEC.md)
//! 4. Credential status/validity check — `Validator::check_credential`
//! 5. Counter/replay check             — `Validator::check_counter`
//! 6. Value/policy checks              — `Validator::check_value`
//! 7. Payee/merchant checks            — `Validator::check_merchant`
//! 8. Persist local evidence           — handled by ledger layer (not here)
//! 9. Expose result to user            — handled by application layer (not here)
//!
//! # Design
//!
//! `Validator` is a pure function object. It takes an envelope, a credential, and
//! a context (current timestamp, known merchant, last-seen counter) and returns a
//! `ValidationResult`. It has no side effects and no I/O.
//!
//! Signature verification (step 3) is intentionally absent here. It requires the
//! crypto boundary (see ADR-002, CRYPTO_SPEC.md). The caller must verify the
//! signature before calling this validator. Calling this validator without prior
//! signature verification is a protocol defect.

use crate::{
    credential::OfflineCredential, envelope::PaymentEnvelopeCore, errors::ValidationError,
    types::MerchantId,
};

/// Outcome of the deterministic validation sequence.
///
/// Note: `Accepted` does NOT mean the transaction is fully settled. It means
/// the deterministic protocol checks passed. Reconciliation is a separate step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationResult {
    /// All checked invariants passed. Proceed to next step.
    ///
    /// **Important:** This does not include signature verification (which must
    /// be done by the caller before invoking the validator).
    Accepted,
    /// One or more invariants failed. The specific reason is attached.
    Rejected(ValidationError),
}

impl ValidationResult {
    #[must_use]
    pub fn is_accepted(&self) -> bool {
        matches!(self, Self::Accepted)
    }

    #[must_use]
    pub fn is_rejected(&self) -> bool {
        matches!(self, Self::Rejected(_))
    }

    /// Return the rejection error, if any.
    #[must_use]
    pub fn rejection_error(&self) -> Option<&ValidationError> {
        match self {
            Self::Rejected(e) => Some(e),
            Self::Accepted => None,
        }
    }
}

/// Context provided to the validator at validation time.
///
/// This separates the pure validation logic from I/O (e.g., database lookups).
/// The caller is responsible for fetching and providing this context before
/// calling `Validator::validate`.
#[derive(Debug)]
pub struct ValidationContext {
    /// Current Unix timestamp in seconds (used for expiry checks).
    pub now_unix_secs: i64,

    /// The expected merchant identity for this validation session.
    /// The envelope's `merchant_id` must match this exactly.
    pub expected_merchant_id: MerchantId,

    /// The last successfully processed counter value for this credential.
    /// A new transaction's counter must be strictly greater than this value.
    ///
    /// Set to `0` if no prior transaction has been processed for this credential
    /// (counters start at 1, so any `counter >= 1` passes when `last_counter = 0`).
    pub last_seen_counter: u64,
}

/// Pure, stateless validator for a `PaymentEnvelopeCore`.
///
/// All methods take immutable references and return results. No side effects.
pub struct Validator;

impl Validator {
    /// Run the complete deterministic validation sequence (steps 4–7).
    ///
    /// **The caller MUST have verified the signature before calling this method.**
    /// Calling without prior signature verification is a protocol defect.
    ///
    /// Returns `ValidationResult::Accepted` if all checks pass.
    /// Returns `ValidationResult::Rejected` on the first failing check.
    ///
    /// Checks are applied in the order specified by PAYMENT_PROTOCOL.md.
    pub fn validate(
        envelope: &PaymentEnvelopeCore,
        credential: &OfflineCredential,
        context: &ValidationContext,
    ) -> ValidationResult {
        // Step 4a: Credential IDs must match.
        if let Err(e) = Self::check_credential_id_matches(envelope, credential) {
            return ValidationResult::Rejected(e);
        }

        // Step 4b: Credential lifecycle and expiry.
        if let Err(e) = Self::check_credential(credential, context.now_unix_secs) {
            return ValidationResult::Rejected(e);
        }

        // Step 5: Counter/replay check.
        if let Err(e) = Self::check_counter(envelope, credential, context.last_seen_counter) {
            return ValidationResult::Rejected(e);
        }

        // Step 6: Value/policy checks.
        if let Err(e) = Self::check_value(envelope, credential) {
            return ValidationResult::Rejected(e);
        }

        // Step 7: Merchant/payee check.
        if let Err(e) = Self::check_merchant(envelope, context.expected_merchant_id) {
            return ValidationResult::Rejected(e);
        }

        // Step 9 (expiry): envelope-level timestamp check.
        if let Err(e) = Self::check_expiry(envelope, context.now_unix_secs) {
            return ValidationResult::Rejected(e);
        }

        ValidationResult::Accepted
    }

    /// Check that the envelope's credential_id matches the provided credential.
    fn check_credential_id_matches(
        envelope: &PaymentEnvelopeCore,
        credential: &OfflineCredential,
    ) -> Result<(), ValidationError> {
        if envelope.credential_id() != credential.credential_id() {
            return Err(ValidationError::FieldOutOfRange {
                field: "credential_id".into(),
                reason: "envelope credential_id does not match provided credential".into(),
            });
        }
        Ok(())
    }

    /// Check credential lifecycle state and expiry.
    ///
    /// Per OFFLINE_CREDENTIAL_SPEC.md §3: only ACTIVE credentials may be used.
    pub fn check_credential(
        credential: &OfflineCredential,
        now_unix_secs: i64,
    ) -> Result<(), ValidationError> {
        if !credential.is_active_at(now_unix_secs) {
            if credential.is_expired_at(now_unix_secs) {
                return Err(ValidationError::TransactionExpired);
            }
            return Err(ValidationError::FieldOutOfRange {
                field: "credential.lifecycle_state".into(),
                reason: format!(
                    "credential is not active (state: {:?})",
                    credential.lifecycle_state()
                ),
            });
        }
        Ok(())
    }

    /// Check the counter/replay invariant.
    ///
    /// The counter must be:
    /// - strictly greater than `last_seen_counter`
    /// - within the credential's `max_counter` bound
    ///
    /// Per CRYPTO_SPEC.md §7: timestamps are NOT the sole replay defense.
    /// Replay is prevented by the counter + credential_id combination.
    pub fn check_counter(
        envelope: &PaymentEnvelopeCore,
        credential: &OfflineCredential,
        last_seen_counter: u64,
    ) -> Result<(), ValidationError> {
        let counter = envelope.counter();

        if counter <= last_seen_counter {
            return Err(ValidationError::CounterViolation {
                reason: format!(
                    "counter {counter} must be strictly greater than last seen {last_seen_counter}"
                ),
            });
        }

        if counter > credential.max_counter() {
            return Err(ValidationError::CounterViolation {
                reason: format!(
                    "counter {counter} exceeds credential max_counter {}",
                    credential.max_counter()
                ),
            });
        }

        Ok(())
    }

    /// Check that the transaction amount does not exceed the credential's per-tx limit.
    pub fn check_value(
        envelope: &PaymentEnvelopeCore,
        credential: &OfflineCredential,
    ) -> Result<(), ValidationError> {
        if !envelope
            .amount()
            .is_within_limit(credential.max_value_per_tx())
        {
            return Err(ValidationError::FieldOutOfRange {
                field: "amount".into(),
                reason: format!(
                    "amount {} {} exceeds per-tx limit {} {}",
                    envelope.amount().amount_minor(),
                    envelope.amount().currency(),
                    credential.max_value_per_tx().amount_minor(),
                    credential.max_value_per_tx().currency(),
                ),
            });
        }
        Ok(())
    }

    /// Check that the envelope's merchant_id matches the expected merchant.
    pub fn check_merchant(
        envelope: &PaymentEnvelopeCore,
        expected_merchant_id: MerchantId,
    ) -> Result<(), ValidationError> {
        if envelope.merchant_id() != expected_merchant_id {
            return Err(ValidationError::FieldOutOfRange {
                field: "merchant_id".into(),
                reason: "envelope merchant_id does not match expected merchant".into(),
            });
        }
        Ok(())
    }

    /// Check that the envelope has not expired.
    pub fn check_expiry(
        envelope: &PaymentEnvelopeCore,
        now_unix_secs: i64,
    ) -> Result<(), ValidationError> {
        if envelope.is_expired_at(now_unix_secs) {
            return Err(ValidationError::TransactionExpired);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        credential::{CredentialLifecycleState, OfflineCredential},
        envelope::{test_fixtures::VALID_NONCE, EnvelopeBuilder, PaymentEnvelopeCore},
        money::Money,
        types::{CredentialId, IssuerId, KeyId, MerchantId, TransactionId},
        PROTOCOL_VERSION,
    };

    /// Build a standard valid credential for tests.
    fn valid_credential(
        cred_id: CredentialId,
        key_id: KeyId,
        merchant_id: MerchantId,
    ) -> OfflineCredential {
        let _ = merchant_id; // not on credential; provided for test clarity
        OfflineCredential::new(
            cred_id,
            key_id,
            IssuerId::generate(),
            1_000_000,
            2_000_000,
            Money::new(50_000, "INR").unwrap(),
            Money::new(200_000, "INR").unwrap(),
            100,
            CredentialLifecycleState::Active,
            1,
        )
        .unwrap()
    }

    /// Build a minimal valid envelope for the given credential/merchant/counter.
    fn valid_envelope(
        cred_id: CredentialId,
        key_id: KeyId,
        merchant_id: MerchantId,
        counter: u64,
        amount_minor: u64,
    ) -> PaymentEnvelopeCore {
        EnvelopeBuilder::new()
            .protocol_version(PROTOCOL_VERSION)
            .tx_id(TransactionId::generate())
            .credential_id(cred_id)
            .payer_key_id(key_id)
            .merchant_id(merchant_id)
            .amount(Money::new(amount_minor, "INR").unwrap())
            .counter(counter)
            .nonce(VALID_NONCE)
            .created_at_unix_secs(1_000_000)
            .expires_at_unix_secs(1_500_000)
            .build()
            .unwrap()
    }

    fn valid_context(merchant_id: MerchantId, last_counter: u64) -> ValidationContext {
        ValidationContext {
            now_unix_secs: 1_200_000,
            expected_merchant_id: merchant_id,
            last_seen_counter: last_counter,
        }
    }

    // -----------------------------------------------------------------------
    // Happy path
    // -----------------------------------------------------------------------

    #[test]
    fn valid_envelope_and_credential_accepted() {
        let cred_id = CredentialId::generate();
        let key_id = KeyId::generate();
        let merchant_id = MerchantId::generate();
        let cred = valid_credential(cred_id, key_id, merchant_id);
        let env = valid_envelope(cred_id, key_id, merchant_id, 1, 10_000);
        let ctx = valid_context(merchant_id, 0);
        assert_eq!(
            Validator::validate(&env, &cred, &ctx),
            ValidationResult::Accepted
        );
    }

    // -----------------------------------------------------------------------
    // Credential ID mismatch
    // -----------------------------------------------------------------------

    #[test]
    fn mismatched_credential_id_rejected() {
        let key_id = KeyId::generate();
        let merchant_id = MerchantId::generate();
        let cred = valid_credential(CredentialId::generate(), key_id, merchant_id);
        // Different credential ID in envelope
        let env = valid_envelope(CredentialId::generate(), key_id, merchant_id, 1, 100);
        let ctx = valid_context(merchant_id, 0);
        let result = Validator::validate(&env, &cred, &ctx);
        assert!(result.is_rejected());
    }

    // -----------------------------------------------------------------------
    // Credential lifecycle
    // -----------------------------------------------------------------------

    #[test]
    fn suspended_credential_rejected() {
        let cred_id = CredentialId::generate();
        let key_id = KeyId::generate();
        let merchant_id = MerchantId::generate();
        let cred = OfflineCredential::new(
            cred_id,
            key_id,
            IssuerId::generate(),
            1_000_000,
            2_000_000,
            Money::new(50_000, "INR").unwrap(),
            Money::new(200_000, "INR").unwrap(),
            100,
            CredentialLifecycleState::Suspended, // not active
            1,
        )
        .unwrap();
        let env = valid_envelope(cred_id, key_id, merchant_id, 1, 100);
        let ctx = valid_context(merchant_id, 0);
        let result = Validator::validate(&env, &cred, &ctx);
        assert!(result.is_rejected());
    }

    #[test]
    fn expired_credential_rejected() {
        let cred_id = CredentialId::generate();
        let key_id = KeyId::generate();
        let merchant_id = MerchantId::generate();
        let cred = valid_credential(cred_id, key_id, merchant_id);
        let env = valid_envelope(cred_id, key_id, merchant_id, 1, 100);
        let ctx = ValidationContext {
            now_unix_secs: 2_000_001, // after credential expiry
            expected_merchant_id: merchant_id,
            last_seen_counter: 0,
        };
        let result = Validator::validate(&env, &cred, &ctx);
        assert!(result.is_rejected());
    }

    // -----------------------------------------------------------------------
    // Counter / replay
    // -----------------------------------------------------------------------

    #[test]
    fn counter_replay_rejected() {
        let cred_id = CredentialId::generate();
        let key_id = KeyId::generate();
        let merchant_id = MerchantId::generate();
        let cred = valid_credential(cred_id, key_id, merchant_id);
        let env = valid_envelope(cred_id, key_id, merchant_id, 5, 100);
        let ctx = ValidationContext {
            now_unix_secs: 1_200_000,
            expected_merchant_id: merchant_id,
            last_seen_counter: 5, // same as envelope — replay
        };
        let result = Validator::validate(&env, &cred, &ctx);
        assert!(result.is_rejected());
        assert!(matches!(
            result.rejection_error(),
            Some(ValidationError::CounterViolation { .. })
        ));
    }

    #[test]
    fn counter_below_last_seen_rejected() {
        let cred_id = CredentialId::generate();
        let key_id = KeyId::generate();
        let merchant_id = MerchantId::generate();
        let cred = valid_credential(cred_id, key_id, merchant_id);
        let env = valid_envelope(cred_id, key_id, merchant_id, 3, 100);
        let ctx = ValidationContext {
            now_unix_secs: 1_200_000,
            expected_merchant_id: merchant_id,
            last_seen_counter: 10, // env counter (3) < last seen (10)
        };
        let result = Validator::validate(&env, &cred, &ctx);
        assert!(result.is_rejected());
    }

    #[test]
    fn counter_exceeds_max_rejected() {
        let cred_id = CredentialId::generate();
        let key_id = KeyId::generate();
        let merchant_id = MerchantId::generate();
        let cred = valid_credential(cred_id, key_id, merchant_id); // max_counter = 100
        let env = valid_envelope(cred_id, key_id, merchant_id, 101, 100); // exceeds 100
        let ctx = valid_context(merchant_id, 0);
        let result = Validator::validate(&env, &cred, &ctx);
        assert!(result.is_rejected());
    }

    // -----------------------------------------------------------------------
    // Value policy
    // -----------------------------------------------------------------------

    #[test]
    fn amount_exceeds_per_tx_limit_rejected() {
        let cred_id = CredentialId::generate();
        let key_id = KeyId::generate();
        let merchant_id = MerchantId::generate();
        let cred = valid_credential(cred_id, key_id, merchant_id); // max_value_per_tx = 50000
        let env = valid_envelope(cred_id, key_id, merchant_id, 1, 50_001); // exceeds
        let ctx = valid_context(merchant_id, 0);
        let result = Validator::validate(&env, &cred, &ctx);
        assert!(result.is_rejected());
        assert!(matches!(
            result.rejection_error(),
            Some(ValidationError::FieldOutOfRange { ref field, .. }) if field == "amount"
        ));
    }

    // -----------------------------------------------------------------------
    // Merchant check
    // -----------------------------------------------------------------------

    #[test]
    fn wrong_merchant_rejected() {
        let cred_id = CredentialId::generate();
        let key_id = KeyId::generate();
        let merchant_id = MerchantId::generate();
        let cred = valid_credential(cred_id, key_id, merchant_id);
        let env = valid_envelope(cred_id, key_id, merchant_id, 1, 100);
        let ctx = ValidationContext {
            now_unix_secs: 1_200_000,
            expected_merchant_id: MerchantId::generate(), // different merchant
            last_seen_counter: 0,
        };
        let result = Validator::validate(&env, &cred, &ctx);
        assert!(result.is_rejected());
    }

    // -----------------------------------------------------------------------
    // Envelope expiry
    // -----------------------------------------------------------------------

    #[test]
    fn expired_envelope_rejected() {
        let cred_id = CredentialId::generate();
        let key_id = KeyId::generate();
        let merchant_id = MerchantId::generate();
        let cred = valid_credential(cred_id, key_id, merchant_id);
        let env = valid_envelope(cred_id, key_id, merchant_id, 1, 100); // expires_at = 1_500_000
        let ctx = ValidationContext {
            now_unix_secs: 1_500_001, // after envelope expiry
            expected_merchant_id: merchant_id,
            last_seen_counter: 0,
        };
        let result = Validator::validate(&env, &cred, &ctx);
        assert!(result.is_rejected());
        assert!(matches!(
            result.rejection_error(),
            Some(ValidationError::TransactionExpired)
        ));
    }

    // -----------------------------------------------------------------------
    // ValidationResult API
    // -----------------------------------------------------------------------

    #[test]
    fn accepted_result_api() {
        assert!(ValidationResult::Accepted.is_accepted());
        assert!(!ValidationResult::Accepted.is_rejected());
        assert!(ValidationResult::Accepted.rejection_error().is_none());
    }

    #[test]
    fn rejected_result_api() {
        let r = ValidationResult::Rejected(ValidationError::TransactionExpired);
        assert!(!r.is_accepted());
        assert!(r.is_rejected());
        assert!(r.rejection_error().is_some());
    }
}
