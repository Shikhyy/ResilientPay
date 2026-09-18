use resilientpay_core::crypto::{Ed25519TestSigner, Signer};
use resilientpay_core::ffi::{AndroidKeyManager, FfiError, ResilientPayClient};

struct MockKeyManager {
    expected_key_id: String,
    expected_payload: Vec<u8>,
    public_key: Vec<u8>,
    mock_signature: Vec<u8>,
}

impl AndroidKeyManager for MockKeyManager {
    fn get_public_key(&self, _key_id: String) -> Result<Vec<u8>, FfiError> {
        Ok(self.public_key.clone())
    }

    fn sign(&self, key_id: String, payload: Vec<u8>) -> Result<Vec<u8>, FfiError> {
        assert_eq!(key_id, self.expected_key_id, "Key ID mismatch crossing FFI");
        assert_eq!(
            payload, self.expected_payload,
            "Signing payload mismatch crossing FFI"
        );
        Ok(self.mock_signature.clone())
    }
}

const VECTOR_TX_UUID: &str = "00000000-0000-0000-0000-000000000001";
const VECTOR_CR_UUID: &str = "00000000-0000-0000-0000-000000000002";
const VECTOR_KEY_UUID: &str = "00000000-0000-0000-0000-000000000003";
const VECTOR_MER_UUID: &str = "00000000-0000-0000-0000-000000000004";
const VECTOR_NONCE: [u8; 16] = [0x01u8; 16];
const VECTOR_AMOUNT: u64 = 150;
const VECTOR_COUNTER: u64 = 1;
const VECTOR_CREATED: i64 = 1_700_000_000;
const VECTOR_EXPIRES: i64 = 1_700_003_600;

// The frozen CBOR (without the domain separator)
const EXPECTED_CBOR_HEX: &str = "8d015000000000000000000000000000000001500000000000000000000000000000000250000000000000000000000000000000035000000000000000000000000000000004189663494e520150010101010101010101010101010101011a6553f1001a6553ff10f6f6";

#[test]
fn test_ffi_create_and_verify_transaction_matches_frozen_vector() {
    let mut expected_payload = b"resilientpay:payment-envelope:v1:".to_vec();
    let cbor_bytes = hex::decode(EXPECTED_CBOR_HEX).unwrap();
    expected_payload.extend_from_slice(&cbor_bytes);

    let signer = Ed25519TestSigner::from_seed(&[0x42; 32]);
    let public_key = signer.public_key().to_bytes().to_vec();
    let signature = signer.sign(&expected_payload).unwrap().to_bytes().to_vec();

    let key_manager = Box::new(MockKeyManager {
        expected_key_id: VECTOR_KEY_UUID.to_string(),
        expected_payload,
        public_key: public_key.clone(),
        mock_signature: signature.clone(),
    });

    let client = ResilientPayClient::new(key_manager);

    let result = client
        .create_transaction(
            VECTOR_TX_UUID.to_string(),
            VECTOR_CR_UUID.to_string(),
            VECTOR_KEY_UUID.to_string(),
            VECTOR_MER_UUID.to_string(),
            VECTOR_AMOUNT,
            VECTOR_COUNTER,
            VECTOR_NONCE.to_vec(),
            VECTOR_CREATED,
            VECTOR_EXPIRES,
        )
        .expect("create_transaction should succeed with local verification");

    let result_json = String::from_utf8(result.clone()).unwrap();

    let sig_hex = hex::encode(&signature);
    assert!(
        result_json.contains(EXPECTED_CBOR_HEX),
        "CBOR hex missing from output JSON"
    );
    assert!(
        result_json.contains(&sig_hex),
        "Signature hex missing from output JSON"
    );

    // Test merchant verification via FFI
    let verified = client
        .verify_transaction(result, public_key)
        .expect("verify_transaction should succeed");
    assert!(verified, "Merchant FFI verification must return true");
}

#[test]
fn test_ffi_invalid_uuid() {
    struct DummyKeyManager;
    impl AndroidKeyManager for DummyKeyManager {
        fn get_public_key(&self, _key_id: String) -> Result<Vec<u8>, FfiError> {
            Ok(vec![])
        }
        fn sign(&self, _key_id: String, _payload: Vec<u8>) -> Result<Vec<u8>, FfiError> {
            Ok(vec![])
        }
    }

    let client = ResilientPayClient::new(Box::new(DummyKeyManager));
    let err = client
        .create_transaction(
            "not-a-uuid".to_string(),
            VECTOR_CR_UUID.to_string(),
            VECTOR_KEY_UUID.to_string(),
            VECTOR_MER_UUID.to_string(),
            VECTOR_AMOUNT,
            VECTOR_COUNTER,
            VECTOR_NONCE.to_vec(),
            VECTOR_CREATED,
            VECTOR_EXPIRES,
        )
        .unwrap_err();

    assert!(matches!(err, FfiError::InvalidInput(_)));
}

#[test]
fn test_ffi_counter_monotonicity_rejected() {
    struct DynamicKeyManager {
        signer: Ed25519TestSigner,
    }
    impl AndroidKeyManager for DynamicKeyManager {
        fn get_public_key(&self, _key_id: String) -> Result<Vec<u8>, FfiError> {
            Ok(self.signer.public_key().to_bytes().to_vec())
        }
        fn sign(&self, _key_id: String, payload: Vec<u8>) -> Result<Vec<u8>, FfiError> {
            Ok(self.signer.sign(&payload).unwrap().to_bytes().to_vec())
        }
    }

    let signer = Ed25519TestSigner::from_seed(&[0x42; 32]);
    let key_manager = Box::new(DynamicKeyManager { signer });
    let client = ResilientPayClient::new(key_manager);

    // First transaction with counter = 10 succeeds
    let _ = client
        .create_transaction(
            VECTOR_TX_UUID.to_string(),
            VECTOR_CR_UUID.to_string(),
            VECTOR_KEY_UUID.to_string(),
            VECTOR_MER_UUID.to_string(),
            VECTOR_AMOUNT,
            10,
            VECTOR_NONCE.to_vec(),
            VECTOR_CREATED,
            VECTOR_EXPIRES,
        )
        .expect("initial transaction should succeed");

    // Second transaction with equal counter (10) must fail
    let err_equal = client
        .create_transaction(
            VECTOR_TX_UUID.to_string(),
            VECTOR_CR_UUID.to_string(),
            VECTOR_KEY_UUID.to_string(),
            VECTOR_MER_UUID.to_string(),
            VECTOR_AMOUNT,
            10,
            VECTOR_NONCE.to_vec(),
            VECTOR_CREATED,
            VECTOR_EXPIRES,
        )
        .unwrap_err();

    assert!(matches!(err_equal, FfiError::InvalidInput(msg) if msg.contains("Counter monotonicity violation")));

    // Third transaction with lower counter (9) must also fail
    let err_lower = client
        .create_transaction(
            VECTOR_TX_UUID.to_string(),
            VECTOR_CR_UUID.to_string(),
            VECTOR_KEY_UUID.to_string(),
            VECTOR_MER_UUID.to_string(),
            VECTOR_AMOUNT,
            9,
            VECTOR_NONCE.to_vec(),
            VECTOR_CREATED,
            VECTOR_EXPIRES,
        )
        .unwrap_err();

    assert!(matches!(err_lower, FfiError::InvalidInput(msg) if msg.contains("Counter monotonicity violation")));
}
