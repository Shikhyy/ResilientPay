//! Offline credential model.
//!
//! An `OfflineCredential` is a bounded authorization artifact associated with
//! a device/key pair. It is NOT a bank credential and MUST NOT contain a UPI PIN.
//!
//! # Protocol reference
//!
//! `docs/05-protocol/OFFLINE_CREDENTIAL_SPEC.md`
//!
//! # Lifecycle
//!
//! ```text
//! REQUESTED → ISSUED → ACTIVE → SUSPENDED → REVOKED
//!                              ↘ EXPIRED
//! ```
//!
//! # Security notes
//!
//! - The credential contains policy limits but NOT private key material.
//! - Expiry is not the sole revocation mechanism — the backend can revoke at any time.
//! - Device loss must trigger backend revocation; the new device must not inherit
//!   the old local state.

use crate::{
    errors::ValidationError,
    money::Money,
    types::{CredentialId, IssuerId, KeyId},
};

/// Credential lifecycle state, per OFFLINE_CREDENTIAL_SPEC.md §3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CredentialLifecycleState {
    /// The credential has been requested but not yet issued.
    Requested,
    /// Issued by the authority; not yet activated.
    Issued,
    /// Active and valid for authorizing transactions.
    Active,
    /// Temporarily suspended (e.g., unusual activity). Not usable.
    Suspended,
    /// Permanently revoked. Terminal state.
    Revoked,
    /// Passed its `expires_at` timestamp. Terminal state.
    Expired,
}

impl CredentialLifecycleState {
    /// Return true if the credential may be used to authorize a transaction.
    #[must_use]
    pub fn is_usable(&self) -> bool {
        matches!(self, Self::Active)
    }

    /// Return true if no further lifecycle transitions are meaningful.
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Revoked | Self::Expired)
    }
}

/// An offline authorization credential.
///
/// Holds policy limits and lifecycle state. Does NOT hold private key material.
/// Private keys remain in platform secure storage (Android Keystore).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfflineCredential {
    credential_id: CredentialId,
    /// Reference to the device signing key associated with this credential.
    subject_key_id: KeyId,
    issuer_id: IssuerId,
    /// Unix timestamp (seconds) when the credential was issued.
    issued_at_unix_secs: i64,
    /// Unix timestamp (seconds) after which this credential MUST be rejected.
    expires_at_unix_secs: i64,
    /// Maximum value per individual transaction (inclusive).
    max_value_per_tx: Money,
    /// Maximum aggregate outstanding (unreconciled) value.
    max_value_outstanding: Money,
    /// Maximum counter value allowed for this credential.
    /// The counter must not exceed this bound.
    max_counter: u64,
    /// Current lifecycle state.
    lifecycle_state: CredentialLifecycleState,
    /// Protocol policy version this credential was issued under.
    policy_version: u32,
}

impl OfflineCredential {
    /// Construct a new `OfflineCredential`.
    ///
    /// # Errors
    ///
    /// Returns `ValidationError` if:
    /// - `expires_at` is not strictly after `issued_at`
    /// - `max_counter` is zero
    // All fields are required by the protocol specification (OFFLINE_CREDENTIAL_SPEC.md §2).
    // A builder would obscure the field-to-spec traceability at no safety benefit.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        credential_id: CredentialId,
        subject_key_id: KeyId,
        issuer_id: IssuerId,
        issued_at_unix_secs: i64,
        expires_at_unix_secs: i64,
        max_value_per_tx: Money,
        max_value_outstanding: Money,
        max_counter: u64,
        lifecycle_state: CredentialLifecycleState,
        policy_version: u32,
    ) -> Result<Self, ValidationError> {
        if expires_at_unix_secs <= issued_at_unix_secs {
            return Err(ValidationError::FieldOutOfRange {
                field: "expires_at_unix_secs".into(),
                reason: "credential expiry must be strictly after issuance".into(),
            });
        }
        if max_counter == 0 {
            return Err(ValidationError::FieldOutOfRange {
                field: "max_counter".into(),
                reason: "max_counter must be >= 1".into(),
            });
        }
        // The per-tx limit must not exceed the outstanding limit (policy sense-check).
        if !max_value_per_tx.is_within_limit(&max_value_outstanding) {
            return Err(ValidationError::FieldOutOfRange {
                field: "max_value_per_tx".into(),
                reason: "per-tx limit must not exceed outstanding limit".into(),
            });
        }
        Ok(Self {
            credential_id,
            subject_key_id,
            issuer_id,
            issued_at_unix_secs,
            expires_at_unix_secs,
            max_value_per_tx,
            max_value_outstanding,
            max_counter,
            lifecycle_state,
            policy_version,
        })
    }

    #[must_use]
    pub fn credential_id(&self) -> CredentialId {
        self.credential_id
    }
    #[must_use]
    pub fn subject_key_id(&self) -> KeyId {
        self.subject_key_id
    }
    #[must_use]
    pub fn issuer_id(&self) -> IssuerId {
        self.issuer_id
    }
    #[must_use]
    pub fn issued_at_unix_secs(&self) -> i64 {
        self.issued_at_unix_secs
    }
    #[must_use]
    pub fn expires_at_unix_secs(&self) -> i64 {
        self.expires_at_unix_secs
    }
    #[must_use]
    pub fn max_value_per_tx(&self) -> &Money {
        &self.max_value_per_tx
    }
    #[must_use]
    pub fn max_value_outstanding(&self) -> &Money {
        &self.max_value_outstanding
    }
    #[must_use]
    pub fn max_counter(&self) -> u64 {
        self.max_counter
    }
    #[must_use]
    pub fn lifecycle_state(&self) -> CredentialLifecycleState {
        self.lifecycle_state
    }
    #[must_use]
    pub fn policy_version(&self) -> u32 {
        self.policy_version
    }

    /// Check if the credential is expired at the given Unix timestamp.
    #[must_use]
    pub fn is_expired_at(&self, now_unix_secs: i64) -> bool {
        now_unix_secs >= self.expires_at_unix_secs
    }

    /// Return `true` if the credential is in a state that permits authorizing
    /// a transaction AND has not expired at `now_unix_secs`.
    #[must_use]
    pub fn is_active_at(&self, now_unix_secs: i64) -> bool {
        self.lifecycle_state.is_usable() && !self.is_expired_at(now_unix_secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn valid_credential() -> OfflineCredential {
        OfflineCredential::new(
            CredentialId::generate(),
            KeyId::generate(),
            IssuerId::generate(),
            1_000_000,
            1_086_400,                           // 1 day later
            Money::new(50_000, "INR").unwrap(),  // ₹500 per tx
            Money::new(200_000, "INR").unwrap(), // ₹2000 outstanding
            100,
            CredentialLifecycleState::Active,
            1,
        )
        .unwrap()
    }

    #[test]
    fn active_credential_is_usable() {
        let cred = valid_credential();
        assert!(cred.lifecycle_state().is_usable());
        assert!(cred.is_active_at(1_000_001));
    }

    #[test]
    fn expired_credential_is_not_active() {
        let cred = valid_credential();
        assert!(!cred.is_active_at(1_086_400)); // at or after expiry
    }

    #[test]
    fn suspended_is_not_usable() {
        assert!(!CredentialLifecycleState::Suspended.is_usable());
    }

    #[test]
    fn revoked_is_terminal() {
        assert!(CredentialLifecycleState::Revoked.is_terminal());
    }

    #[test]
    fn expiry_not_after_issuance_rejected() {
        let err = OfflineCredential::new(
            CredentialId::generate(),
            KeyId::generate(),
            IssuerId::generate(),
            1_000_000,
            1_000_000, // equal — not strictly after
            Money::new(100, "INR").unwrap(),
            Money::new(100, "INR").unwrap(),
            10,
            CredentialLifecycleState::Active,
            1,
        )
        .unwrap_err();
        assert!(matches!(err, ValidationError::FieldOutOfRange { .. }));
    }

    #[test]
    fn zero_max_counter_rejected() {
        let err = OfflineCredential::new(
            CredentialId::generate(),
            KeyId::generate(),
            IssuerId::generate(),
            1_000_000,
            1_086_400,
            Money::new(100, "INR").unwrap(),
            Money::new(100, "INR").unwrap(),
            0, // zero max_counter
            CredentialLifecycleState::Active,
            1,
        )
        .unwrap_err();
        assert!(
            matches!(err, ValidationError::FieldOutOfRange { ref field, .. } if field == "max_counter")
        );
    }

    #[test]
    fn per_tx_exceeding_outstanding_rejected() {
        let err = OfflineCredential::new(
            CredentialId::generate(),
            KeyId::generate(),
            IssuerId::generate(),
            1_000_000,
            1_086_400,
            Money::new(200_000, "INR").unwrap(), // per-tx > outstanding
            Money::new(100_000, "INR").unwrap(),
            10,
            CredentialLifecycleState::Active,
            1,
        )
        .unwrap_err();
        assert!(
            matches!(err, ValidationError::FieldOutOfRange { ref field, .. } if field == "max_value_per_tx")
        );
    }
}
