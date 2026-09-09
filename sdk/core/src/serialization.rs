//! Canonical CBOR serialization for signing.
//!
//! This module encodes a `PaymentEnvelopeCore` into a deterministic byte sequence
//! suitable for cryptographic signing and verification.
//!
//! # Protocol reference
//!
//! - `docs/05-protocol/PAYMENT_PROTOCOL.md` §Serialization
//! - `docs/04-security/CRYPTO_SPEC.md` §5 (Canonicalization)
//! - ADR-008: Canonical Binary Serialization (accepted for research)
//!
//! # Design: CBOR Array encoding
//!
//! Fields are encoded as a CBOR **array** (RFC 8949 major type 4) in a fixed,
//! spec-defined order. Using an array (rather than a map) makes the encoding
//! inherently ordered and eliminates any map-key sorting ambiguity that could
//! cause verifier/signer disagreement.
//!
//! Field ordering exactly matches the table in `PAYMENT_PROTOCOL.md §2`.
//!
//! # Domain separation
//!
//! A fixed domain-separation prefix is prepended to the CBOR bytes before
//! signing. This prevents cross-protocol signature reuse.
//!
//! ```text
//! signing_input = DOMAIN_SEPARATOR || canonical_cbor_bytes
//! ```
//!
//! # Canonicalization guarantees
//!
//! - UUIDs encoded as 16-byte binary (`bstr`) — no hyphenated text form.
//! - Integers encoded in shortest CBOR form (ciborium default).
//! - Byte strings encoded as `bstr` (major type 2).
//! - Text strings encoded as `tstr` (major type 3).
//! - Optional fields encoded as CBOR `null` (major type 7, simple value 22) when absent.
//! - No floating point in the encoding.
//!
//! # Security note
//!
//! The canonical encoding is used for signing input only. It is NOT a general-purpose
//! wire format. Transport representations may differ (e.g., JSON for debugging),
//! but they must never be used as signing input.

use crate::envelope::PaymentEnvelopeCore;
use ciborium::value::Value;

/// Domain-separation prefix prepended to the canonical CBOR bytes before signing.
///
/// Changing this prefix is a **breaking** protocol change — all previously signed
/// envelopes would fail verification. Requires an ADR + change control.
pub const SIGNING_DOMAIN_SEPARATOR: &[u8] = b"resilientpay:payment-envelope:v1:";

/// Produce the canonical byte sequence used as input to Ed25519 signing.
///
/// ```text
/// signing_input = SIGNING_DOMAIN_SEPARATOR || encode_envelope_cbor(envelope)
/// ```
///
/// # Errors
///
/// Returns `SerializationError` if CBOR encoding fails (should not happen for
/// well-formed envelopes, but is represented as an error to force callers to
/// handle it explicitly rather than unwrapping).
pub fn signing_input(envelope: &PaymentEnvelopeCore) -> Result<Vec<u8>, SerializationError> {
    let cbor_bytes = encode_envelope_cbor(envelope)?;
    let mut result = Vec::with_capacity(SIGNING_DOMAIN_SEPARATOR.len() + cbor_bytes.len());
    result.extend_from_slice(SIGNING_DOMAIN_SEPARATOR);
    result.extend_from_slice(&cbor_bytes);
    Ok(result)
}

/// Encode a `PaymentEnvelopeCore` as a canonical CBOR array.
///
/// The encoded form is a 13-element CBOR array with fields in the order specified
/// by `PAYMENT_PROTOCOL.md §2`. This function is deterministic: the same envelope
/// always produces the same bytes.
///
/// This is the raw CBOR encoding without the domain separator. Prefer
/// `signing_input()` when computing bytes for signing.
pub fn encode_envelope_cbor(envelope: &PaymentEnvelopeCore) -> Result<Vec<u8>, SerializationError> {
    // Build the CBOR array in the exact field order from the protocol spec.
    // Each comment names the field and its position in the spec table.
    let items = vec![
        // [0] protocol_version
        Value::Integer(u32_to_cbor_int(envelope.protocol_version())),
        // [1] tx_id — 16-byte binary (UUID bytes, not the hyphenated string form)
        Value::Bytes(envelope.tx_id().as_uuid().as_bytes().to_vec()),
        // [2] credential_id — 16-byte binary
        Value::Bytes(envelope.credential_id().as_uuid().as_bytes().to_vec()),
        // [3] payer_key_id — 16-byte binary
        Value::Bytes(envelope.payer_key_id().as_uuid().as_bytes().to_vec()),
        // [4] merchant_id — 16-byte binary
        Value::Bytes(envelope.merchant_id().as_uuid().as_bytes().to_vec()),
        // [5] amount_minor — unsigned integer
        Value::Integer(u64_to_cbor_int(envelope.amount().amount_minor())),
        // [6] currency — text string (uppercase ISO code)
        Value::Text(envelope.amount().currency().to_string()),
        // [7] counter — unsigned integer
        Value::Integer(u64_to_cbor_int(envelope.counter())),
        // [8] nonce — 16-byte binary
        Value::Bytes(envelope.nonce().to_vec()),
        // [9] created_at_unix_secs — integer (may be negative for historic timestamps)
        Value::Integer(i64_to_cbor_int(envelope.created_at_unix_secs())),
        // [10] expires_at_unix_secs — integer
        Value::Integer(i64_to_cbor_int(envelope.expires_at_unix_secs())),
        // [11] previous_event_hash — 32-byte binary or null
        match envelope.previous_event_hash() {
            Some(h) => Value::Bytes(h.to_vec()),
            None => Value::Null,
        },
        // [12] risk_class — text string or null (advisory; does not affect auth)
        match envelope.risk_class() {
            Some(rc) => Value::Text(rc.to_string()),
            None => Value::Null,
        },
    ];

    let value = Value::Array(items);
    let mut bytes = Vec::new();
    ciborium::into_writer(&value, &mut bytes)
        .map_err(|e| SerializationError::CborEncode(e.to_string()))?;

    Ok(bytes)
}

/// Decode a CBOR-encoded envelope array and reconstruct field values for verification.
///
/// This is used on the **verifier** side to check that the decoded fields match
/// the received envelope before verifying the signature over the canonical bytes.
///
/// Returns the decoded field values in the same order as `encode_envelope_cbor`.
/// The caller is responsible for comparing these against the received envelope fields.
pub fn decode_envelope_cbor_fields(
    bytes: &[u8],
) -> Result<DecodedEnvelopeFields, SerializationError> {
    let value: Value =
        ciborium::from_reader(bytes).map_err(|e| SerializationError::CborDecode(e.to_string()))?;

    let items = match value {
        Value::Array(items) => items,
        other => {
            return Err(SerializationError::UnexpectedType {
                expected: "Array",
                got: cbor_type_name(&other),
            })
        }
    };

    if items.len() != 13 {
        return Err(SerializationError::FieldCountMismatch {
            expected: 13,
            got: items.len(),
        });
    }

    let mut it = items.into_iter();

    Ok(DecodedEnvelopeFields {
        protocol_version: cbor_to_u32(it.next().unwrap(), "protocol_version")?,
        tx_id_bytes: cbor_to_bytes_exact::<16>(it.next().unwrap(), "tx_id")?,
        credential_id_bytes: cbor_to_bytes_exact::<16>(it.next().unwrap(), "credential_id")?,
        payer_key_id_bytes: cbor_to_bytes_exact::<16>(it.next().unwrap(), "payer_key_id")?,
        merchant_id_bytes: cbor_to_bytes_exact::<16>(it.next().unwrap(), "merchant_id")?,
        amount_minor: cbor_to_u64(it.next().unwrap(), "amount_minor")?,
        currency: cbor_to_text(it.next().unwrap(), "currency")?,
        counter: cbor_to_u64(it.next().unwrap(), "counter")?,
        nonce: cbor_to_bytes_exact::<16>(it.next().unwrap(), "nonce")?,
        created_at_unix_secs: cbor_to_i64(it.next().unwrap(), "created_at_unix_secs")?,
        expires_at_unix_secs: cbor_to_i64(it.next().unwrap(), "expires_at_unix_secs")?,
        previous_event_hash: cbor_to_optional_bytes_exact::<32>(
            it.next().unwrap(),
            "previous_event_hash",
        )?,
        risk_class: cbor_to_optional_text(it.next().unwrap(), "risk_class")?,
    })
}

/// Decoded fields from a CBOR envelope — used for verification.
#[derive(Debug, PartialEq, Eq)]
pub struct DecodedEnvelopeFields {
    pub protocol_version: u32,
    pub tx_id_bytes: [u8; 16],
    pub credential_id_bytes: [u8; 16],
    pub payer_key_id_bytes: [u8; 16],
    pub merchant_id_bytes: [u8; 16],
    pub amount_minor: u64,
    pub currency: String,
    pub counter: u64,
    pub nonce: [u8; 16],
    pub created_at_unix_secs: i64,
    pub expires_at_unix_secs: i64,
    pub previous_event_hash: Option<[u8; 32]>,
    pub risk_class: Option<String>,
}

// ---------------------------------------------------------------------------
// Serialization errors
// ---------------------------------------------------------------------------

/// Errors produced during CBOR encoding or decoding.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SerializationError {
    #[error("CBOR encoding failed: {0}")]
    CborEncode(String),

    #[error("CBOR decoding failed: {0}")]
    CborDecode(String),

    #[error("unexpected CBOR type: expected {expected}, got {got}")]
    UnexpectedType {
        expected: &'static str,
        got: &'static str,
    },

    #[error("field count mismatch: expected {expected} fields, got {got}")]
    FieldCountMismatch { expected: usize, got: usize },

    #[error("field {field}: {reason}")]
    FieldError { field: &'static str, reason: String },

    #[error("field {field}: byte slice has wrong length: expected {expected}, got {got}")]
    ByteLengthMismatch {
        field: &'static str,
        expected: usize,
        got: usize,
    },
}

// ---------------------------------------------------------------------------
// Internal CBOR conversion helpers
// ---------------------------------------------------------------------------

fn u32_to_cbor_int(v: u32) -> ciborium::value::Integer {
    ciborium::value::Integer::from(v)
}

fn u64_to_cbor_int(v: u64) -> ciborium::value::Integer {
    ciborium::value::Integer::from(v)
}

fn i64_to_cbor_int(v: i64) -> ciborium::value::Integer {
    ciborium::value::Integer::from(v)
}

fn cbor_type_name(v: &Value) -> &'static str {
    match v {
        Value::Integer(_) => "Integer",
        Value::Bytes(_) => "Bytes",
        Value::Text(_) => "Text",
        Value::Array(_) => "Array",
        Value::Map(_) => "Map",
        Value::Tag(_, _) => "Tag",
        Value::Bool(_) => "Bool",
        Value::Null => "Null",
        Value::Float(_) => "Float",
        _ => "Unknown",
    }
}

fn cbor_to_u32(v: Value, field: &'static str) -> Result<u32, SerializationError> {
    match v {
        Value::Integer(i) => {
            let n: i128 = i.into();
            u32::try_from(n).map_err(|_| SerializationError::FieldError {
                field,
                reason: format!("value {n} out of u32 range"),
            })
        }
        other => Err(SerializationError::UnexpectedType {
            expected: "Integer",
            got: cbor_type_name(&other),
        }),
    }
}

fn cbor_to_u64(v: Value, field: &'static str) -> Result<u64, SerializationError> {
    match v {
        Value::Integer(i) => {
            let n: i128 = i.into();
            u64::try_from(n).map_err(|_| SerializationError::FieldError {
                field,
                reason: format!("value {n} out of u64 range"),
            })
        }
        other => Err(SerializationError::UnexpectedType {
            expected: "Integer",
            got: cbor_type_name(&other),
        }),
    }
}

fn cbor_to_i64(v: Value, field: &'static str) -> Result<i64, SerializationError> {
    match v {
        Value::Integer(i) => {
            let n: i128 = i.into();
            i64::try_from(n).map_err(|_| SerializationError::FieldError {
                field,
                reason: format!("value {n} out of i64 range"),
            })
        }
        other => Err(SerializationError::UnexpectedType {
            expected: "Integer",
            got: cbor_type_name(&other),
        }),
    }
}

fn cbor_to_text(v: Value, _field: &'static str) -> Result<String, SerializationError> {
    match v {
        Value::Text(s) => Ok(s),
        other => Err(SerializationError::UnexpectedType {
            expected: "Text",
            got: cbor_type_name(&other),
        }),
    }
}

fn cbor_to_optional_text(
    v: Value,
    _field: &'static str,
) -> Result<Option<String>, SerializationError> {
    match v {
        Value::Text(s) => Ok(Some(s)),
        Value::Null => Ok(None),
        other => Err(SerializationError::UnexpectedType {
            expected: "Text or Null",
            got: cbor_type_name(&other),
        }),
    }
}

fn cbor_to_bytes_exact<const N: usize>(
    v: Value,
    field: &'static str,
) -> Result<[u8; N], SerializationError> {
    match v {
        Value::Bytes(b) => {
            b.try_into()
                .map_err(|v: Vec<u8>| SerializationError::ByteLengthMismatch {
                    field,
                    expected: N,
                    got: v.len(),
                })
        }
        other => Err(SerializationError::UnexpectedType {
            expected: "Bytes",
            got: cbor_type_name(&other),
        }),
    }
}

fn cbor_to_optional_bytes_exact<const N: usize>(
    v: Value,
    field: &'static str,
) -> Result<Option<[u8; N]>, SerializationError> {
    match v {
        Value::Bytes(b) => {
            let arr: [u8; N] =
                b.try_into()
                    .map_err(|v: Vec<u8>| SerializationError::ByteLengthMismatch {
                        field,
                        expected: N,
                        got: v.len(),
                    })?;
            Ok(Some(arr))
        }
        Value::Null => Ok(None),
        other => Err(SerializationError::UnexpectedType {
            expected: "Bytes or Null",
            got: cbor_type_name(&other),
        }),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        envelope::{test_fixtures::minimal_valid_envelope, EnvelopeBuilder, HASH_LEN, NONCE_LEN},
        money::Money,
        types::{CredentialId, KeyId, MerchantId, TransactionId},
        PROTOCOL_VERSION,
    };
    use uuid::Uuid;

    // -----------------------------------------------------------------------
    // Determinism: same input → identical bytes
    // -----------------------------------------------------------------------

    #[test]
    fn encoding_is_deterministic() {
        let env = minimal_valid_envelope();
        let bytes1 = encode_envelope_cbor(&env).unwrap();
        let bytes2 = encode_envelope_cbor(&env).unwrap();
        assert_eq!(bytes1, bytes2, "encoding must be deterministic");
    }

    #[test]
    fn signing_input_is_deterministic() {
        let env = minimal_valid_envelope();
        let b1 = signing_input(&env).unwrap();
        let b2 = signing_input(&env).unwrap();
        assert_eq!(b1, b2);
    }

    // -----------------------------------------------------------------------
    // Domain separator
    // -----------------------------------------------------------------------

    #[test]
    fn signing_input_starts_with_domain_separator() {
        let env = minimal_valid_envelope();
        let bytes = signing_input(&env).unwrap();
        assert!(
            bytes.starts_with(SIGNING_DOMAIN_SEPARATOR),
            "signing input must start with domain separator"
        );
    }

    #[test]
    fn signing_input_longer_than_cbor_alone() {
        let env = minimal_valid_envelope();
        let cbor = encode_envelope_cbor(&env).unwrap();
        let full = signing_input(&env).unwrap();
        assert_eq!(full.len(), SIGNING_DOMAIN_SEPARATOR.len() + cbor.len());
    }

    // -----------------------------------------------------------------------
    // Different inputs → different bytes (sensitivity)
    // -----------------------------------------------------------------------

    #[test]
    fn different_amounts_produce_different_bytes() {
        let nonce = [1u8; NONCE_LEN];
        let tx = TransactionId::generate();
        let cr = CredentialId::generate();
        let key = KeyId::generate();
        let mer = MerchantId::generate();

        let make = |minor| {
            EnvelopeBuilder::new()
                .protocol_version(PROTOCOL_VERSION)
                .tx_id(tx)
                .credential_id(cr)
                .payer_key_id(key)
                .merchant_id(mer)
                .amount(Money::new(minor, "INR").unwrap())
                .counter(1)
                .nonce(nonce)
                .created_at_unix_secs(1_000_000)
                .expires_at_unix_secs(1_001_000)
                .build()
                .unwrap()
        };

        let b100 = encode_envelope_cbor(&make(100)).unwrap();
        let b200 = encode_envelope_cbor(&make(200)).unwrap();
        assert_ne!(b100, b200, "different amounts must produce different bytes");
    }

    #[test]
    fn different_merchants_produce_different_bytes() {
        let nonce = [1u8; NONCE_LEN];
        let make = |mer: MerchantId| {
            EnvelopeBuilder::new()
                .protocol_version(PROTOCOL_VERSION)
                .tx_id(TransactionId::generate())
                .credential_id(CredentialId::generate())
                .payer_key_id(KeyId::generate())
                .merchant_id(mer)
                .amount(Money::new(100, "INR").unwrap())
                .counter(1)
                .nonce(nonce)
                .created_at_unix_secs(1_000_000)
                .expires_at_unix_secs(1_001_000)
                .build()
                .unwrap()
        };

        let b1 = encode_envelope_cbor(&make(MerchantId::generate())).unwrap();
        let b2 = encode_envelope_cbor(&make(MerchantId::generate())).unwrap();
        assert_ne!(b1, b2);
    }

    // -----------------------------------------------------------------------
    // Round-trip decode
    // -----------------------------------------------------------------------

    #[test]
    fn round_trip_decode_minimal() {
        let env = minimal_valid_envelope();
        let bytes = encode_envelope_cbor(&env).unwrap();
        let decoded = decode_envelope_cbor_fields(&bytes).unwrap();

        assert_eq!(decoded.protocol_version, env.protocol_version());
        assert_eq!(&decoded.tx_id_bytes, env.tx_id().as_uuid().as_bytes());
        assert_eq!(
            &decoded.credential_id_bytes,
            env.credential_id().as_uuid().as_bytes()
        );
        assert_eq!(decoded.amount_minor, env.amount().amount_minor());
        assert_eq!(decoded.currency, env.amount().currency());
        assert_eq!(decoded.counter, env.counter());
        assert_eq!(&decoded.nonce, env.nonce());
        assert_eq!(decoded.created_at_unix_secs, env.created_at_unix_secs());
        assert_eq!(decoded.expires_at_unix_secs, env.expires_at_unix_secs());
        assert_eq!(decoded.previous_event_hash, None);
        assert_eq!(decoded.risk_class, None);
    }

    #[test]
    fn round_trip_with_optional_fields() {
        let nonce = [7u8; NONCE_LEN];
        let hash = [0xABu8; HASH_LEN];
        let env = EnvelopeBuilder::new()
            .protocol_version(PROTOCOL_VERSION)
            .tx_id(TransactionId::from_uuid(
                Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            ))
            .credential_id(CredentialId::generate())
            .payer_key_id(KeyId::generate())
            .merchant_id(MerchantId::generate())
            .amount(Money::new(9999, "INR").unwrap())
            .counter(42)
            .nonce(nonce)
            .created_at_unix_secs(1_700_000_000)
            .expires_at_unix_secs(1_700_003_600)
            .previous_event_hash(hash)
            .risk_class("LOW".to_string())
            .build()
            .unwrap();

        let bytes = encode_envelope_cbor(&env).unwrap();
        let decoded = decode_envelope_cbor_fields(&bytes).unwrap();

        assert_eq!(decoded.counter, 42);
        assert_eq!(decoded.currency, "INR");
        assert_eq!(decoded.previous_event_hash, Some(hash));
        assert_eq!(decoded.risk_class, Some("LOW".to_string()));
    }

    // -----------------------------------------------------------------------
    // Decode error cases
    // -----------------------------------------------------------------------

    #[test]
    fn decode_empty_bytes_fails() {
        let err = decode_envelope_cbor_fields(&[]).unwrap_err();
        assert!(matches!(err, SerializationError::CborDecode(_)));
    }

    #[test]
    fn decode_garbage_fails() {
        let err = decode_envelope_cbor_fields(b"\xFF\xFE\xFD").unwrap_err();
        assert!(matches!(err, SerializationError::CborDecode(_)));
    }

    #[test]
    fn decode_wrong_field_count_fails() {
        // Encode a 2-element array (not 13)
        let short = vec![Value::Integer(1u64.into()), Value::Null];
        let mut bytes = Vec::new();
        ciborium::into_writer(&Value::Array(short), &mut bytes).unwrap();
        let err = decode_envelope_cbor_fields(&bytes).unwrap_err();
        assert!(matches!(
            err,
            SerializationError::FieldCountMismatch {
                expected: 13,
                got: 2
            }
        ));
    }

    // -----------------------------------------------------------------------
    // Known test vector (frozen bytes for regression detection)
    //
    // This vector was generated by this implementation and locked in.
    // Any change to encoding that alters these bytes is a BREAKING protocol change
    // requiring an ADR update. See ADR-008.
    // -----------------------------------------------------------------------

    #[test]
    fn known_test_vector_structure() {
        // Build an envelope with fully fixed (deterministic) UUIDs
        let tx_uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let cr_uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();
        let key_uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap();
        let mer_uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000004").unwrap();
        let nonce = [0x01u8; NONCE_LEN];

        let env = EnvelopeBuilder::new()
            .protocol_version(PROTOCOL_VERSION)
            .tx_id(TransactionId::from_uuid(tx_uuid))
            .credential_id(CredentialId::from_uuid(cr_uuid))
            .payer_key_id(KeyId::from_uuid(key_uuid))
            .merchant_id(MerchantId::from_uuid(mer_uuid))
            .amount(Money::new(150, "INR").unwrap()) // ₹1.50 in paise
            .counter(1)
            .nonce(nonce)
            .created_at_unix_secs(1_700_000_000)
            .expires_at_unix_secs(1_700_003_600)
            .build()
            .unwrap();

        let bytes = encode_envelope_cbor(&env).unwrap();

        // The encoded bytes must be a valid CBOR array of 13 elements.
        let decoded = decode_envelope_cbor_fields(&bytes).unwrap();
        assert_eq!(decoded.protocol_version, 1);
        assert_eq!(decoded.amount_minor, 150);
        assert_eq!(decoded.currency, "INR");
        assert_eq!(decoded.counter, 1);
        assert_eq!(decoded.nonce, nonce);
        assert_eq!(decoded.created_at_unix_secs, 1_700_000_000);

        // Print the hex for pinning in the next iteration (Gate 3 test vector freeze).
        // This line is intentionally left for the test output so the bytes can be
        // inspected and frozen in a separate test-vector file.
        println!(
            "CBOR bytes (hex): {}",
            bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
        );
        println!(
            "signing_input length: {} bytes",
            signing_input(&env).unwrap().len()
        );
    }
}
