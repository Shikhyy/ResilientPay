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
}
