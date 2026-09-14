use resilientpay_core::ffi::{AndroidKeyManager, FfiError, ResilientPayClient};

struct MockKeyManager {
    expected_key_id: String,
    expected_payload: Vec<u8>,
    mock_signature: Vec<u8>,
}

impl AndroidKeyManager for MockKeyManager {
    fn get_public_key(&self, _key_id: String) -> Result<Vec<u8>, FfiError> {
        Ok(vec![])
    }

    fn sign(&self, key_id: String, payload: Vec<u8>) -> Result<Vec<u8>, FfiError> {
        assert_eq!(key_id, self.expected_key_id, "Key ID mismatch crossing FFI");
        assert_eq!(payload, self.expected_payload, "Signing payload mismatch crossing FFI");
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
fn test_ffi_create_transaction_matches_frozen_vector() {
    let mut expected_payload = b"resilientpay:payment-envelope:v1:".to_vec();
    let cbor_bytes = hex::decode(EXPECTED_CBOR_HEX).unwrap();
    expected_payload.extend_from_slice(&cbor_bytes);

    let mock_signature = vec![0x99; 64];

    let key_manager = Box::new(MockKeyManager {
        expected_key_id: VECTOR_KEY_UUID.to_string(),
        expected_payload,
        mock_signature: mock_signature.clone(),
    });

    let client = ResilientPayClient::new(key_manager);

    let result = client.create_transaction(
        VECTOR_TX_UUID.to_string(),
        VECTOR_CR_UUID.to_string(),
        VECTOR_KEY_UUID.to_string(),
        VECTOR_MER_UUID.to_string(),
        VECTOR_AMOUNT,
        VECTOR_COUNTER,
        VECTOR_NONCE.to_vec(),
        VECTOR_CREATED,
        VECTOR_EXPIRES,
    ).expect("create_transaction should succeed");

    let result_json = String::from_utf8(result).unwrap();
    
    let mock_sig_hex = hex::encode(mock_signature);
    assert!(result_json.contains(EXPECTED_CBOR_HEX), "CBOR hex missing from output JSON");
    assert!(result_json.contains(&mock_sig_hex), "Signature hex missing from output JSON");
}

#[test]
fn test_ffi_invalid_uuid() {
    struct DummyKeyManager;
    impl AndroidKeyManager for DummyKeyManager {
        fn get_public_key(&self, _key_id: String) -> Result<Vec<u8>, FfiError> { Ok(vec![]) }
        fn sign(&self, _key_id: String, _payload: Vec<u8>) -> Result<Vec<u8>, FfiError> { Ok(vec![]) }
    }

    let client = ResilientPayClient::new(Box::new(DummyKeyManager));
    let err = client.create_transaction(
        "not-a-uuid".to_string(),
        VECTOR_CR_UUID.to_string(),
        VECTOR_KEY_UUID.to_string(),
        VECTOR_MER_UUID.to_string(),
        VECTOR_AMOUNT,
        VECTOR_COUNTER,
        VECTOR_NONCE.to_vec(),
        VECTOR_CREATED,
        VECTOR_EXPIRES,
    ).unwrap_err();

    assert!(matches!(err, FfiError::InvalidInput(_)));
}
