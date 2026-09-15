use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

use crate::envelope::PaymentEnvelopeCore;
use crate::errors::ValidationError;
use crate::money::Money;
use crate::serialization::signing_input;
use crate::types::{CredentialId, KeyId, MerchantId, TransactionId};

#[derive(Debug, Error, uniffi::Error)]
pub enum FfiError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Hardware signing failed: {0}")]
    SigningFailure(String),
    #[error("Key unavailable: {0}")]
    KeyUnavailable(String),
    #[error("Serialization failure: {0}")]
    SerializationFailure(String),
    #[error("Internal SDK error: {0}")]
    Internal(String),
}

impl From<ValidationError> for FfiError {
    fn from(e: ValidationError) -> Self {
        FfiError::InvalidInput(e.to_string())
    }
}

#[uniffi::export(callback_interface)]
pub trait AndroidKeyManager: Send + Sync {
    /// Retrieve the public key bytes for a given key_id
    fn get_public_key(&self, key_id: String) -> Result<Vec<u8>, FfiError>;

    /// Request the hardware keystore to sign the exact payload
    fn sign(&self, key_id: String, payload: Vec<u8>) -> Result<Vec<u8>, FfiError>;
}

#[derive(uniffi::Object)]
pub struct ResilientPayClient {
    key_manager: Box<dyn AndroidKeyManager>,
}

#[uniffi::export]
impl ResilientPayClient {
    #[uniffi::constructor]
    pub fn new(key_manager: Box<dyn AndroidKeyManager>) -> Arc<Self> {
        Arc::new(Self { key_manager })
    }

    /// Creates a transaction. Prepares canonical bytes -> calls hardware sign -> returns signed envelope payload.
    #[allow(clippy::too_many_arguments)]
    pub fn create_transaction(
        &self,
        tx_id_str: String,
        credential_id_str: String,
        payer_key_id_str: String,
        merchant_id_str: String,
        amount_minor: u64,
        counter: u64,
        nonce_bytes: Vec<u8>,
        created_at_unix: i64,
        expires_at_unix: i64,
    ) -> Result<Vec<u8>, FfiError> {
        let tx_id = TransactionId::from_uuid(
            Uuid::parse_str(&tx_id_str).map_err(|e| FfiError::InvalidInput(e.to_string()))?,
        );
        let credential_id = CredentialId::from_uuid(
            Uuid::parse_str(&credential_id_str)
                .map_err(|e| FfiError::InvalidInput(e.to_string()))?,
        );
        let payer_key_id = KeyId::from_uuid(
            Uuid::parse_str(&payer_key_id_str)
                .map_err(|e| FfiError::InvalidInput(e.to_string()))?,
        );
        let merchant_id = MerchantId::from_uuid(
            Uuid::parse_str(&merchant_id_str).map_err(|e| FfiError::InvalidInput(e.to_string()))?,
        );

        let nonce_array: [u8; 16] = nonce_bytes
            .try_into()
            .map_err(|_| FfiError::InvalidInput("Nonce must be exactly 16 bytes".to_string()))?;

        let amount = Money::new(amount_minor, "INR")?;

        // 1. Build the domain object (PaymentEnvelopeCore)
        let core = PaymentEnvelopeCore::new(
            1, // protocol_version
            tx_id,
            credential_id,
            payer_key_id,
            merchant_id,
            amount,
            counter,
            nonce_array,
            created_at_unix,
            expires_at_unix,
            None, // previous_event_hash
            None, // risk_class
        )?;

        // 2 & 3. Canonical serialization + domain separation
        let signing_input_bytes =
            signing_input(&core).map_err(|e| FfiError::SerializationFailure(e.to_string()))?;

        // 4. Request signing from the Android Keystore callback
        let signature_bytes = self
            .key_manager
            .sign(payer_key_id_str.clone(), signing_input_bytes.clone())?;

        // Ensure signature is 64 bytes
        if signature_bytes.len() != 64 {
            return Err(FfiError::SigningFailure(format!(
                "Hardware signature was {} bytes, expected 64",
                signature_bytes.len()
            )));
        }

        // Verify signature locally using the public key from the key manager (ADR-012 §4 Step 6)
        let pubkey_bytes = self
            .key_manager
            .get_public_key(payer_key_id_str)?;
        let verifier = crate::crypto::Ed25519Verifier::from_bytes(&pubkey_bytes)
            .map_err(|e| FfiError::SigningFailure(format!("Invalid public key: {}", e)))?;
        let sig = crate::crypto::Signature::from_bytes(&signature_bytes)
            .map_err(|e| FfiError::SigningFailure(format!("Invalid signature format: {}", e)))?;
        crate::crypto::verify_envelope(&core, &sig, &verifier)
            .map_err(|e| FfiError::SigningFailure(format!("Local verification failed: {}", e)))?;

        // 5. Build final payload representation.
        let core_cbor = crate::serialization::encode_envelope_cbor(&core)
            .map_err(|e| FfiError::SerializationFailure(e.to_string()))?;

        let hex_cbor: String = core_cbor.iter().map(|b| format!("{:02x}", b)).collect();
        let hex_sig: String = signature_bytes
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();

        let envelope_json = format!(
            r#"{{"core_cbor_hex":"{}","signature_hex":"{}"}}"#,
            hex_cbor, hex_sig
        );

        Ok(envelope_json.into_bytes())
    }

    /// Verifies a received payment envelope payload against a trusted payer public key.
    /// Returns true if the signature is valid.
    pub fn verify_transaction(
        &self,
        envelope_json_bytes: Vec<u8>,
        payer_public_key: Vec<u8>,
    ) -> Result<bool, FfiError> {
        let json_str = String::from_utf8(envelope_json_bytes)
            .map_err(|e| FfiError::InvalidInput(format!("Invalid UTF-8: {}", e)))?;

        let (cbor_hex, sig_hex) = parse_envelope_json(&json_str)?;
        let cbor_bytes = hex::decode(&cbor_hex)
            .map_err(|e| FfiError::InvalidInput(format!("Invalid CBOR hex: {}", e)))?;
        let sig_bytes = hex::decode(&sig_hex)
            .map_err(|e| FfiError::InvalidInput(format!("Invalid signature hex: {}", e)))?;

        let decoded = crate::serialization::decode_envelope_cbor_fields(&cbor_bytes)
            .map_err(|e| FfiError::SerializationFailure(e.to_string()))?;

        let tx_id = TransactionId::from_uuid(Uuid::from_bytes(decoded.tx_id_bytes));
        let credential_id = CredentialId::from_uuid(Uuid::from_bytes(decoded.credential_id_bytes));
        let payer_key_id = KeyId::from_uuid(Uuid::from_bytes(decoded.payer_key_id_bytes));
        let merchant_id = MerchantId::from_uuid(Uuid::from_bytes(decoded.merchant_id_bytes));
        let amount = Money::new(decoded.amount_minor, &decoded.currency)?;

        let core = PaymentEnvelopeCore::new(
            decoded.protocol_version,
            tx_id,
            credential_id,
            payer_key_id,
            merchant_id,
            amount,
            decoded.counter,
            decoded.nonce,
            decoded.created_at_unix_secs,
            decoded.expires_at_unix_secs,
            decoded.previous_event_hash,
            decoded.risk_class,
        )?;

        let verifier = crate::crypto::Ed25519Verifier::from_bytes(&payer_public_key)
            .map_err(|e| FfiError::InvalidInput(format!("Invalid public key: {}", e)))?;
        let sig = crate::crypto::Signature::from_bytes(&sig_bytes)
            .map_err(|e| FfiError::InvalidInput(format!("Invalid signature format: {}", e)))?;

        match crate::crypto::verify_envelope(&core, &sig, &verifier) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

fn parse_envelope_json(json_str: &str) -> Result<(String, String), FfiError> {
    let cbor_prefix = "\"core_cbor_hex\":\"";
    let sig_prefix = "\"signature_hex\":\"";

    let cbor_start = json_str
        .find(cbor_prefix)
        .ok_or_else(|| FfiError::InvalidInput("Missing core_cbor_hex in envelope JSON".into()))?
        + cbor_prefix.len();
    let cbor_end = json_str[cbor_start..]
        .find('"')
        .ok_or_else(|| FfiError::InvalidInput("Unterminated core_cbor_hex string".into()))?
        + cbor_start;
    let cbor_hex = &json_str[cbor_start..cbor_end];

    let sig_start = json_str
        .find(sig_prefix)
        .ok_or_else(|| FfiError::InvalidInput("Missing signature_hex in envelope JSON".into()))?
        + sig_prefix.len();
    let sig_end = json_str[sig_start..]
        .find('"')
        .ok_or_else(|| FfiError::InvalidInput("Unterminated signature_hex string".into()))?
        + sig_start;
    let sig_hex = &json_str[sig_start..sig_end];

    Ok((cbor_hex.to_string(), sig_hex.to_string()))
}
