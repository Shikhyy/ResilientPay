//! Payment envelope core — the signed record of a payment.
//!
//! `PaymentEnvelopeCore` contains every field that MUST be included in the
//! canonical byte sequence before signing. It does not contain the signature
//! itself (which is held by the outer transport/wire envelope).
//!
//! # Protocol reference
//!
//! `docs/05-protocol/PAYMENT_PROTOCOL.md` §2 (Logical envelope)
//! `docs/04-security/CRYPTO_SPEC.md` §4 (Signed fields)
//!
//! # Field table (normative)
//!
//! | Field                | Type          | Required | Notes                                    |
//! |----------------------|---------------|----------|------------------------------------------|
//! | protocol_version     | u32           | yes      | Must match PROTOCOL_VERSION constant     |
//! | tx_id                | TransactionId | yes      | Unique per transaction                   |
//! | credential_id        | CredentialId  | yes      | Binds to authorizing credential          |
//! | payer_key_id         | KeyId         | yes      | Reference to signing key (NOT key bytes) |
//! | merchant_id          | MerchantId    | yes      | Authenticated payee identity             |
//! | amount               | Money         | yes      | Integer minor units, explicit currency   |
//! | counter              | u64           | yes      | Monotonic, per-credential               |
//! | nonce                | [u8; 16]      | yes      | Random, prevents trivial duplicates      |
//! | created_at_unix_secs | i64           | yes      | Unix timestamp (metadata/expiry only)    |
//! | expires_at_unix_secs | i64           | yes      | Hard expiry; validated against now       |
//! | previous_event_hash  | Option<[u8;32]>| no      | Hash-chain link for ledger integrity     |
//! | risk_class           | Option<String>| no      | Advisory only; does not alter auth       |
//!
//! # Security notes
//!
//! - No private key material appears here.
//! - The `risk_class` field is advisory. It MUST NOT be used to bypass hard
//!   cryptographic authorization (see ADR-010, security-rules.md rule 11).
//! - `previous_event_hash` provides tamper evidence when hash-chaining is active
//!   (ADR-009) but does not by itself prevent local rewriting.

use crate::{
    errors::ValidationError,
    money::Money,
    types::{CredentialId, KeyId, MerchantId, TransactionId},
    PROTOCOL_VERSION,
};

/// The minimum nonce length in bytes.
pub const NONCE_LEN: usize = 16;

/// The hash length used for the previous-event hash chain link (SHA-256).
pub const HASH_LEN: usize = 32;

/// Maximum length of the `risk_class` string (advisory field; bounded to prevent abuse).
const MAX_RISK_CLASS_LEN: usize = 64;

/// Core payment envelope — the set of fields that are canonically encoded and signed.
///
/// Once constructed through `PaymentEnvelopeCore::new`, the struct is immutable.
/// Modification requires building a new envelope (which would require a new signature).
///
/// See the module-level documentation for the normative field table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentEnvelopeCore {
    /// Protocol version. Fail-closed: reject envelopes with unsupported versions.
    protocol_version: u32,

    /// Unique transaction identifier.
    tx_id: TransactionId,

    /// Reference to the offline credential that authorizes this transaction.
    credential_id: CredentialId,

    /// Reference to the payer's signing key (NOT the key material itself).
    payer_key_id: KeyId,

    /// Authenticated merchant/payee identity.
    merchant_id: MerchantId,

    /// Payment amount (integer minor units, explicit currency).
    amount: Money,

    /// Monotonically increasing counter for the authorizing credential.
    /// Combined with `credential_id`, this is the primary replay defense.
    counter: u64,

    /// Random nonce, exactly `NONCE_LEN` bytes.
    nonce: [u8; NONCE_LEN],

    /// Unix timestamp (seconds) when this envelope was created.
    /// This is supporting metadata for expiry and analytics.
    /// It is NOT the sole replay defense (see CRYPTO_SPEC.md §7).
    created_at_unix_secs: i64,

    /// Unix timestamp (seconds) after which this envelope MUST be rejected.
    expires_at_unix_secs: i64,

    /// Optional hash of the previous ledger event, for hash-chain integrity.
    /// `None` means this is the first event in the chain, or hash-chaining is
    /// not active for this envelope profile.
    previous_event_hash: Option<[u8; HASH_LEN]>,

    /// Advisory risk classification label. Does not affect cryptographic authorization.
    risk_class: Option<String>,
}

impl PaymentEnvelopeCore {
    /// Construct and validate a `PaymentEnvelopeCore`.
    ///
    /// All required fields are validated at construction time. Returns a
    /// `ValidationError` if any field is invalid.
    ///
    /// # Errors
    ///
    /// - `ValidationError::UnsupportedProtocolVersion` — version ≠ PROTOCOL_VERSION
    /// - `ValidationError::FieldOutOfRange` — expiry before or equal to created_at
    /// - `ValidationError::InvalidNonce` — nonce is all-zeros (structural check)
    /// - `ValidationError::FieldOutOfRange` — counter is zero (counters start at 1)
    /// - `ValidationError::InvalidCharacters` — risk_class contains disallowed chars
    /// - `ValidationError::MissingField` — risk_class exceeds maximum length
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        protocol_version: u32,
        tx_id: TransactionId,
        credential_id: CredentialId,
        payer_key_id: KeyId,
        merchant_id: MerchantId,
        amount: Money,
        counter: u64,
        nonce: [u8; NONCE_LEN],
        created_at_unix_secs: i64,
        expires_at_unix_secs: i64,
        previous_event_hash: Option<[u8; HASH_LEN]>,
        risk_class: Option<String>,
    ) -> Result<Self, ValidationError> {
        // 1. Protocol version check — fail closed.
        if protocol_version != PROTOCOL_VERSION {
            return Err(ValidationError::UnsupportedProtocolVersion {
                version: protocol_version,
                supported: PROTOCOL_VERSION,
            });
        }

        // 2. Counter must be ≥ 1. Zero is reserved to mean "not assigned".
        if counter == 0 {
            return Err(ValidationError::FieldOutOfRange {
                field: "counter".into(),
                reason: "counter must be >= 1".into(),
            });
        }

        // 3. Expiry must be strictly after creation.
        if expires_at_unix_secs <= created_at_unix_secs {
            return Err(ValidationError::FieldOutOfRange {
                field: "expires_at_unix_secs".into(),
                reason: "expiry must be strictly after creation time".into(),
            });
        }

        // 4. Nonce must not be all-zeros (would indicate uninitialized memory / bug).
        if nonce.iter().all(|&b| b == 0) {
            return Err(ValidationError::InvalidNonce {
                reason: "nonce must not be all-zeros".into(),
            });
        }

        // 5. Optional risk_class: bounded length, ASCII printable only.
        if let Some(ref rc) = risk_class {
            if rc.len() > MAX_RISK_CLASS_LEN {
                return Err(ValidationError::MissingField {
                    field: format!(
                        "risk_class length {} exceeds maximum {}",
                        rc.len(),
                        MAX_RISK_CLASS_LEN
                    ),
                });
            }
            if !rc.chars().all(|c| c.is_ascii() && !c.is_ascii_control()) {
                return Err(ValidationError::InvalidCharacters {
                    field: "risk_class".into(),
                    reason: "must contain only printable ASCII characters".into(),
                });
            }
        }

        Ok(Self {
            protocol_version,
            tx_id,
            credential_id,
            payer_key_id,
            merchant_id,
            amount,
            counter,
            nonce,
            created_at_unix_secs,
            expires_at_unix_secs,
            previous_event_hash,
            risk_class,
        })
    }

    // -----------------------------------------------------------------------
    // Accessors (all read-only — the envelope is immutable after construction)
    // -----------------------------------------------------------------------

    #[must_use]
    pub fn protocol_version(&self) -> u32 {
        self.protocol_version
    }
    #[must_use]
    pub fn tx_id(&self) -> TransactionId {
        self.tx_id
    }
    #[must_use]
    pub fn credential_id(&self) -> CredentialId {
        self.credential_id
    }
    #[must_use]
    pub fn payer_key_id(&self) -> KeyId {
        self.payer_key_id
    }
    #[must_use]
    pub fn merchant_id(&self) -> MerchantId {
        self.merchant_id
    }
    #[must_use]
    pub fn amount(&self) -> &Money {
        &self.amount
    }
    #[must_use]
    pub fn counter(&self) -> u64 {
        self.counter
    }
    #[must_use]
    pub fn nonce(&self) -> &[u8; NONCE_LEN] {
        &self.nonce
    }
    #[must_use]
    pub fn created_at_unix_secs(&self) -> i64 {
        self.created_at_unix_secs
    }
    #[must_use]
    pub fn expires_at_unix_secs(&self) -> i64 {
        self.expires_at_unix_secs
    }
    #[must_use]
    pub fn previous_event_hash(&self) -> Option<&[u8; HASH_LEN]> {
        self.previous_event_hash.as_ref()
    }
    #[must_use]
    pub fn risk_class(&self) -> Option<&str> {
        self.risk_class.as_deref()
    }

    /// Return `true` if the envelope has expired at the given Unix timestamp.
    ///
    /// Note: `now_unix_secs` should come from a trusted source, but timestamp
    /// checks are supporting metadata only — not the sole replay defense.
    #[must_use]
    pub fn is_expired_at(&self, now_unix_secs: i64) -> bool {
        now_unix_secs >= self.expires_at_unix_secs
    }
}

/// Builder for `PaymentEnvelopeCore` — useful in tests and protocol construction code.
///
/// The builder accumulates fields and calls `build()` to validate them all at once.
/// This avoids requiring callers to provide all 12 arguments in a single positional call.
#[derive(Debug, Default)]
pub struct EnvelopeBuilder {
    protocol_version: Option<u32>,
    tx_id: Option<TransactionId>,
    credential_id: Option<CredentialId>,
    payer_key_id: Option<KeyId>,
    merchant_id: Option<MerchantId>,
    amount: Option<Money>,
    counter: Option<u64>,
    nonce: Option<[u8; NONCE_LEN]>,
    created_at_unix_secs: Option<i64>,
    expires_at_unix_secs: Option<i64>,
    previous_event_hash: Option<[u8; HASH_LEN]>,
    risk_class: Option<String>,
}

impl EnvelopeBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    #[must_use]
    pub fn protocol_version(mut self, v: u32) -> Self {
        self.protocol_version = Some(v);
        self
    }
    #[must_use]
    pub fn tx_id(mut self, v: TransactionId) -> Self {
        self.tx_id = Some(v);
        self
    }
    #[must_use]
    pub fn credential_id(mut self, v: CredentialId) -> Self {
        self.credential_id = Some(v);
        self
    }
    #[must_use]
    pub fn payer_key_id(mut self, v: KeyId) -> Self {
        self.payer_key_id = Some(v);
        self
    }
    #[must_use]
    pub fn merchant_id(mut self, v: MerchantId) -> Self {
        self.merchant_id = Some(v);
        self
    }
    #[must_use]
    pub fn amount(mut self, v: Money) -> Self {
        self.amount = Some(v);
        self
    }
    #[must_use]
    pub fn counter(mut self, v: u64) -> Self {
        self.counter = Some(v);
        self
    }
    #[must_use]
    pub fn nonce(mut self, v: [u8; NONCE_LEN]) -> Self {
        self.nonce = Some(v);
        self
    }
    #[must_use]
    pub fn created_at_unix_secs(mut self, v: i64) -> Self {
        self.created_at_unix_secs = Some(v);
        self
    }
    #[must_use]
    pub fn expires_at_unix_secs(mut self, v: i64) -> Self {
        self.expires_at_unix_secs = Some(v);
        self
    }
    #[must_use]
    pub fn previous_event_hash(mut self, v: [u8; HASH_LEN]) -> Self {
        self.previous_event_hash = Some(v);
        self
    }
    #[must_use]
    pub fn risk_class(mut self, v: String) -> Self {
        self.risk_class = Some(v);
        self
    }

    /// Validate all fields and construct the envelope.
    pub fn build(self) -> Result<PaymentEnvelopeCore, ValidationError> {
        let missing = |field: &str| ValidationError::MissingField {
            field: field.into(),
        };

        PaymentEnvelopeCore::new(
            self.protocol_version
                .ok_or_else(|| missing("protocol_version"))?,
            self.tx_id.ok_or_else(|| missing("tx_id"))?,
            self.credential_id.ok_or_else(|| missing("credential_id"))?,
            self.payer_key_id.ok_or_else(|| missing("payer_key_id"))?,
            self.merchant_id.ok_or_else(|| missing("merchant_id"))?,
            self.amount.ok_or_else(|| missing("amount"))?,
            self.counter.ok_or_else(|| missing("counter"))?,
            self.nonce.ok_or_else(|| missing("nonce"))?,
            self.created_at_unix_secs
                .ok_or_else(|| missing("created_at_unix_secs"))?,
            self.expires_at_unix_secs
                .ok_or_else(|| missing("expires_at_unix_secs"))?,
            self.previous_event_hash,
            self.risk_class,
        )
    }
}

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

#[cfg(test)]
pub mod test_fixtures {
    use super::*;
    use crate::types::{CredentialId, KeyId, MerchantId, TransactionId};

    /// A valid nonce for use in tests (non-zero, 16 bytes).
    pub const VALID_NONCE: [u8; NONCE_LEN] =
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

    /// Construct a minimal valid envelope for tests.
    pub fn minimal_valid_envelope() -> PaymentEnvelopeCore {
        PaymentEnvelopeCore::new(
            PROTOCOL_VERSION,
            TransactionId::generate(),
            CredentialId::generate(),
            KeyId::generate(),
            MerchantId::generate(),
            Money::new(100, "INR").unwrap(),
            1,
            VALID_NONCE,
            1_000_000,
            1_000_060, // 60 seconds later
            None,
            None,
        )
        .expect("test fixture must be valid")
    }
}

#[cfg(test)]
mod tests {
    use super::test_fixtures::*;
    use super::*;
    use crate::types::{CredentialId, KeyId, MerchantId, TransactionId};
    use crate::PROTOCOL_VERSION;

    fn base_args() -> (TransactionId, CredentialId, KeyId, MerchantId, Money) {
        (
            TransactionId::generate(),
            CredentialId::generate(),
            KeyId::generate(),
            MerchantId::generate(),
            Money::new(500, "INR").unwrap(),
        )
    }

    #[test]
    fn valid_envelope_constructs() {
        let e = minimal_valid_envelope();
        assert_eq!(e.protocol_version(), PROTOCOL_VERSION);
        assert_eq!(e.counter(), 1);
        assert_eq!(e.amount().amount_minor(), 100);
        assert!(!e.is_expired_at(1_000_059));
        assert!(e.is_expired_at(1_000_060));
    }

    #[test]
    fn unsupported_protocol_version_rejected() {
        let (tx, cr, key, mer, amt) = base_args();
        let err = PaymentEnvelopeCore::new(
            PROTOCOL_VERSION + 1,
            tx,
            cr,
            key,
            mer,
            amt,
            1,
            VALID_NONCE,
            1_000_000,
            1_000_060,
            None,
            None,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            ValidationError::UnsupportedProtocolVersion { .. }
        ));
    }

    #[test]
    fn zero_counter_rejected() {
        let (tx, cr, key, mer, amt) = base_args();
        let err = PaymentEnvelopeCore::new(
            PROTOCOL_VERSION,
            tx,
            cr,
            key,
            mer,
            amt,
            0,
            VALID_NONCE,
            1_000_000,
            1_000_060,
            None,
            None,
        )
        .unwrap_err();
        assert!(
            matches!(err, ValidationError::FieldOutOfRange { ref field, .. } if field == "counter")
        );
    }

    #[test]
    fn expiry_before_creation_rejected() {
        let (tx, cr, key, mer, amt) = base_args();
        let err = PaymentEnvelopeCore::new(
            PROTOCOL_VERSION,
            tx,
            cr,
            key,
            mer,
            amt,
            1,
            VALID_NONCE,
            1_000_060,
            1_000_000,
            None,
            None, // expiry before created_at
        )
        .unwrap_err();
        assert!(
            matches!(err, ValidationError::FieldOutOfRange { ref field, .. } if field == "expires_at_unix_secs")
        );
    }

    #[test]
    fn expiry_equal_to_creation_rejected() {
        let (tx, cr, key, mer, amt) = base_args();
        let err = PaymentEnvelopeCore::new(
            PROTOCOL_VERSION,
            tx,
            cr,
            key,
            mer,
            amt,
            1,
            VALID_NONCE,
            1_000_000,
            1_000_000,
            None,
            None, // equal
        )
        .unwrap_err();
        assert!(matches!(err, ValidationError::FieldOutOfRange { .. }));
    }

    #[test]
    fn all_zero_nonce_rejected() {
        let (tx, cr, key, mer, amt) = base_args();
        let err = PaymentEnvelopeCore::new(
            PROTOCOL_VERSION,
            tx,
            cr,
            key,
            mer,
            amt,
            1,
            [0u8; NONCE_LEN],
            1_000_000,
            1_000_060,
            None,
            None,
        )
        .unwrap_err();
        assert!(matches!(err, ValidationError::InvalidNonce { .. }));
    }

    #[test]
    fn risk_class_too_long_rejected() {
        let (tx, cr, key, mer, amt) = base_args();
        let long = "A".repeat(65);
        let err = PaymentEnvelopeCore::new(
            PROTOCOL_VERSION,
            tx,
            cr,
            key,
            mer,
            amt,
            1,
            VALID_NONCE,
            1_000_000,
            1_000_060,
            None,
            Some(long),
        )
        .unwrap_err();
        assert!(matches!(err, ValidationError::MissingField { .. }));
    }

    #[test]
    fn risk_class_with_control_chars_rejected() {
        let (tx, cr, key, mer, amt) = base_args();
        let err = PaymentEnvelopeCore::new(
            PROTOCOL_VERSION,
            tx,
            cr,
            key,
            mer,
            amt,
            1,
            VALID_NONCE,
            1_000_000,
            1_000_060,
            None,
            Some("low\x01risk".into()),
        )
        .unwrap_err();
        assert!(
            matches!(err, ValidationError::InvalidCharacters { ref field, .. } if field == "risk_class")
        );
    }

    #[test]
    fn optional_fields_can_be_none() {
        let e = minimal_valid_envelope();
        assert!(e.previous_event_hash().is_none());
        assert!(e.risk_class().is_none());
    }

    #[test]
    fn previous_event_hash_stored_correctly() {
        let (tx, cr, key, mer, amt) = base_args();
        let hash = [0xABu8; HASH_LEN];
        let e = PaymentEnvelopeCore::new(
            PROTOCOL_VERSION,
            tx,
            cr,
            key,
            mer,
            amt,
            1,
            VALID_NONCE,
            1_000_000,
            1_000_060,
            Some(hash),
            None,
        )
        .unwrap();
        assert_eq!(e.previous_event_hash(), Some(&hash));
    }

    #[test]
    fn builder_produces_same_result_as_direct_construction() {
        let tx = TransactionId::generate();
        let cr = CredentialId::generate();
        let key = KeyId::generate();
        let mer = MerchantId::generate();
        let amt = Money::new(100, "INR").unwrap();

        let direct = PaymentEnvelopeCore::new(
            PROTOCOL_VERSION,
            tx,
            cr,
            key,
            mer,
            amt.clone(),
            1,
            VALID_NONCE,
            1_000_000,
            1_000_060,
            None,
            None,
        )
        .unwrap();

        let built = EnvelopeBuilder::new()
            .protocol_version(PROTOCOL_VERSION)
            .tx_id(tx)
            .credential_id(cr)
            .payer_key_id(key)
            .merchant_id(mer)
            .amount(amt)
            .counter(1)
            .nonce(VALID_NONCE)
            .created_at_unix_secs(1_000_000)
            .expires_at_unix_secs(1_000_060)
            .build()
            .unwrap();

        assert_eq!(direct, built);
    }

    #[test]
    fn builder_rejects_missing_required_field() {
        let err = EnvelopeBuilder::new()
            .protocol_version(PROTOCOL_VERSION)
            // tx_id deliberately omitted
            .build()
            .unwrap_err();
        assert!(matches!(err, ValidationError::MissingField { ref field } if field == "tx_id"));
    }
}
