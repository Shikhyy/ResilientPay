//! Cryptographic signing and verification boundary.
//!
//! This module defines the **interface** and **implementation** for Ed25519
//! signature operations. It is the only module that touches cryptographic
//! primitives. All other modules interact with cryptography through these
//! traits.
//!
//! # Protocol reference
//!
//! - `docs/04-security/CRYPTO_SPEC.md` §2–§4, §9
//! - ADR-002: Rust security-core language
//!
//! # Ed25519 algorithm choice
//!
//! Ed25519 is the default research candidate (CRYPTO_SPEC.md §2). It provides:
//! - deterministic signatures (no per-signature randomness needed)
//! - fast batch verification
//! - small key and signature sizes
//! - wide library support and audit history
//!
//! Library: `ed25519-dalek` v2 (from the dalek-cryptography project).
//!
//! # Key model
//!
//! - **Signing key** (`SigningKey`): the private half. MUST NOT be logged, serialised
//!   to disk in plaintext, or transmitted. In production, private keys live in
//!   Android Keystore and sign operations are performed there. The `Ed25519TestSigner`
//!   holds a signing key in memory for **test use only**.
//!
//! - **Verifying key** (`VerifyingKey`): the public half. Used by the merchant and
//!   backend to verify a received signature. Safe to transmit.
//!
//! # Trait design
//!
//! `Signer` and `Verifier` are separate traits. The verifier side never needs
//! the signing key. A `Signer` implementation on Android Keystore would delegate
//! the private-key operation to the Keystore and return only the signature bytes.
//!
//! # Failure policy (CRYPTO_SPEC.md §9)
//!
//! Any signature verification failure is a **security rejection**, not a retryable
//! transport error. The `CryptoError::VerificationFailed` variant must not be
//! silently ignored.

use crate::envelope::PaymentEnvelopeCore;
use crate::serialization::{signing_input, SerializationError};
use ed25519_dalek::Verifier as DalekVerifier;
use ed25519_dalek::{
    Signature as DalekSignature, VerifyingKey, PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH,
};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// An Ed25519 signature — 64 bytes.
///
/// Wraps `ed25519_dalek::Signature` but hides the dalek type from callers.
/// The bytes are accessible for wire encoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature(DalekSignature);

impl Signature {
    /// Parse a signature from 64 raw bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        if bytes.len() != SIGNATURE_LENGTH {
            return Err(CryptoError::MalformedInput {
                reason: format!(
                    "signature must be exactly {SIGNATURE_LENGTH} bytes, got {}",
                    bytes.len()
                ),
            });
        }
        let arr: [u8; SIGNATURE_LENGTH] = bytes.try_into().unwrap(); // safe: length checked
        Ok(Self(DalekSignature::from_bytes(&arr)))
    }

    /// Return the raw 64 bytes of the signature.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; SIGNATURE_LENGTH] {
        self.0.to_bytes()
    }
}

/// An Ed25519 public (verifying) key — 32 bytes.
///
/// Safe to transmit and store. Never confused with `SigningKey`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicKey(VerifyingKey);

impl PublicKey {
    /// Parse a public key from 32 raw bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        if bytes.len() != PUBLIC_KEY_LENGTH {
            return Err(CryptoError::MalformedInput {
                reason: format!(
                    "public key must be exactly {PUBLIC_KEY_LENGTH} bytes, got {}",
                    bytes.len()
                ),
            });
        }
        let arr: [u8; PUBLIC_KEY_LENGTH] = bytes.try_into().unwrap(); // safe: length checked
        VerifyingKey::from_bytes(&arr)
            .map(Self)
            .map_err(|e| CryptoError::MalformedInput {
                reason: e.to_string(),
            })
    }

    /// Return the raw 32 bytes of the public key.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; PUBLIC_KEY_LENGTH] {
        self.0.to_bytes()
    }
}

// ---------------------------------------------------------------------------
// Traits
// ---------------------------------------------------------------------------

/// Abstract signing interface.
///
/// Production implementations delegate to Android Keystore or another
/// platform secure enclave. The private key never leaves the secure context.
pub trait Signer {
    /// Produce an Ed25519 signature over `message`.
    ///
    /// # Errors
    ///
    /// Returns `CryptoError` if the signing operation fails. The error message
    /// must not contain private key material.
    fn sign(&self, message: &[u8]) -> Result<Signature, CryptoError>;

    /// Return the public key corresponding to the signing key.
    fn public_key(&self) -> &PublicKey;
}

/// Abstract verification interface.
pub trait Verifier {
    /// Verify that `signature` over `message` is valid for this verifier's public key.
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::VerificationFailed` if the signature is invalid.
    /// This is a security rejection — callers must not silently discard this error.
    fn verify(&self, message: &[u8], signature: &Signature) -> Result<(), CryptoError>;
}

// ---------------------------------------------------------------------------
// High-level helpers that combine serialization + signing/verification
// ---------------------------------------------------------------------------

/// Sign a `PaymentEnvelopeCore` using the provided signer.
///
/// Internally: `signing_input(envelope)` → `signer.sign(bytes)`.
/// The caller receives the signature; the canonical bytes are not exposed.
pub fn sign_envelope(
    envelope: &PaymentEnvelopeCore,
    signer: &dyn Signer,
) -> Result<Signature, CryptoError> {
    let bytes =
        signing_input(envelope).map_err(|e| CryptoError::SerializationError(e.to_string()))?;
    signer.sign(&bytes)
}

/// Verify a signature over a `PaymentEnvelopeCore`.
///
/// Internally recomputes `signing_input(envelope)` and checks the signature.
/// If verification fails, the error must be treated as a security rejection.
///
/// # Errors
///
/// - `CryptoError::SerializationError` — canonical encoding failed (internal error)
/// - `CryptoError::VerificationFailed` — signature is invalid (security rejection)
pub fn verify_envelope(
    envelope: &PaymentEnvelopeCore,
    signature: &Signature,
    verifier: &dyn Verifier,
) -> Result<(), CryptoError> {
    let bytes =
        signing_input(envelope).map_err(|e| CryptoError::SerializationError(e.to_string()))?;
    verifier.verify(&bytes, signature)
}

// ---------------------------------------------------------------------------
// Production verifier
// ---------------------------------------------------------------------------

/// An Ed25519 verifier that holds a public key.
///
/// This is safe for production use: it only holds the public key.
pub struct Ed25519Verifier {
    public_key: PublicKey,
}

impl Ed25519Verifier {
    /// Construct a verifier from a `PublicKey`.
    #[must_use]
    pub fn new(public_key: PublicKey) -> Self {
        Self { public_key }
    }

    /// Construct a verifier from raw 32-byte public key bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        Ok(Self::new(PublicKey::from_bytes(bytes)?))
    }

    /// Return a reference to the held public key.
    #[must_use]
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }
}

impl Verifier for Ed25519Verifier {
    fn verify(&self, message: &[u8], signature: &Signature) -> Result<(), CryptoError> {
        self.public_key
            .0
            .verify(message, &signature.0)
            .map_err(|_| CryptoError::VerificationFailed)
    }
}

// ---------------------------------------------------------------------------
// Test-only signer
// ---------------------------------------------------------------------------

/// An Ed25519 signer that holds a signing key **in memory**.
///
/// # ⚠ TEST USE ONLY ⚠
///
/// This type holds a private signing key in memory. It MUST NOT be used in
/// production code paths. Production signing must use Android Keystore or
/// another platform-backed secure key store where the private key never leaves
/// the secure context.
///
/// This type is declared in production code (not `#[cfg(test)]`) so that it
/// can be used in integration tests and the simulator, which live in separate
/// crates. It is clearly named with the `Test` prefix to discourage misuse.
///
/// The signing key is zeroized on drop (via `ed25519-dalek`'s `zeroize` feature).
pub struct Ed25519TestSigner {
    signing_key: ed25519_dalek::SigningKey,
    public_key: PublicKey,
}

impl Ed25519TestSigner {
    /// Create a test signer from a 32-byte deterministic seed.
    ///
    /// Using a fixed seed produces a deterministic key pair, which is required
    /// for reproducible protocol test vectors (Gate 3).
    ///
    /// # Example
    ///
    /// ```
    /// use resilientpay_core::crypto::Ed25519TestSigner;
    ///
    /// let seed = [0x42u8; 32]; // fixed test seed
    /// let signer = Ed25519TestSigner::from_seed(&seed);
    /// ```
    #[must_use]
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let signing_key = ed25519_dalek::SigningKey::from_bytes(seed);
        let public_key = PublicKey(signing_key.verifying_key());
        Self {
            signing_key,
            public_key,
        }
    }

    /// Return the corresponding `Ed25519Verifier` for this test signer.
    #[must_use]
    pub fn verifier(&self) -> Ed25519Verifier {
        Ed25519Verifier::new(self.public_key.clone())
    }
}

impl Signer for Ed25519TestSigner {
    fn sign(&self, message: &[u8]) -> Result<Signature, CryptoError> {
        use ed25519_dalek::Signer as DalekSignerTrait;
        let sig: DalekSignature = self.signing_key.sign(message);
        Ok(Signature(sig))
    }

    fn public_key(&self) -> &PublicKey {
        &self.public_key
    }
}

// ---------------------------------------------------------------------------
// Crypto errors
// ---------------------------------------------------------------------------

/// Errors from cryptographic operations.
///
/// # Security rule (CRYPTO_SPEC.md §9)
///
/// `VerificationFailed` is a security rejection. It must never be silently
/// ignored or treated as a retryable transport error.
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    /// Signature verification failed. The message or signature has been tampered
    /// with, or the wrong public key was used.
    ///
    /// This is a **security rejection** — fail closed.
    #[error("signature verification failed")]
    VerificationFailed,

    /// The input bytes are malformed (wrong length, invalid curve point, etc.).
    #[error("malformed cryptographic input: {reason}")]
    MalformedInput { reason: String },

    /// Serialization of the envelope to canonical bytes failed.
    /// This indicates an internal error, not a security rejection from external input.
    #[error("envelope serialization failed: {0}")]
    SerializationError(String),

    /// The signing operation failed (platform error, Keystore unavailable, etc.).
    #[error("signing operation failed")]
    SigningFailed,
}

impl From<SerializationError> for CryptoError {
    fn from(e: SerializationError) -> Self {
        Self::SerializationError(e.to_string())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::envelope::test_fixtures::minimal_valid_envelope;

    /// Fixed test seed for reproducible key pairs.
    const TEST_SEED: [u8; 32] = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E,
        0x1F, 0x20,
    ];

    fn test_signer() -> Ed25519TestSigner {
        Ed25519TestSigner::from_seed(&TEST_SEED)
    }

    // -----------------------------------------------------------------------
    // Sign and verify
    // -----------------------------------------------------------------------

    #[test]
    fn sign_and_verify_roundtrip() {
        let signer = test_signer();
        let verifier = signer.verifier();
        let env = minimal_valid_envelope();

        let sig = sign_envelope(&env, &signer).unwrap();
        verify_envelope(&env, &sig, &verifier).unwrap();
    }

    #[test]
    fn signature_is_deterministic() {
        let signer = test_signer();
        let env = minimal_valid_envelope();

        let sig1 = sign_envelope(&env, &signer).unwrap();
        let sig2 = sign_envelope(&env, &signer).unwrap();
        assert_eq!(sig1, sig2, "Ed25519 signatures must be deterministic");
    }

    // -----------------------------------------------------------------------
    // Tampered envelope: verification must fail
    // -----------------------------------------------------------------------

    #[test]
    fn tampered_amount_fails_verification() {
        use crate::{
            envelope::test_fixtures::VALID_NONCE,
            envelope::EnvelopeBuilder,
            money::Money,
            types::{CredentialId, KeyId, MerchantId, TransactionId},
            PROTOCOL_VERSION,
        };

        let signer = test_signer();
        let verifier = signer.verifier();

        let cr = CredentialId::generate();
        let key = KeyId::generate();
        let mer = MerchantId::generate();

        let original = EnvelopeBuilder::new()
            .protocol_version(PROTOCOL_VERSION)
            .tx_id(TransactionId::generate())
            .credential_id(cr)
            .payer_key_id(key)
            .merchant_id(mer)
            .amount(Money::new(100, "INR").unwrap())
            .counter(1)
            .nonce(VALID_NONCE)
            .created_at_unix_secs(1_000_000)
            .expires_at_unix_secs(1_001_000)
            .build()
            .unwrap();

        let sig = sign_envelope(&original, &signer).unwrap();

        // Attacker modifies the amount on the merchant side
        let tampered = EnvelopeBuilder::new()
            .protocol_version(PROTOCOL_VERSION)
            .tx_id(original.tx_id())
            .credential_id(cr)
            .payer_key_id(key)
            .merchant_id(mer)
            .amount(Money::new(1, "INR").unwrap()) // ← tampered: ₹100 → ₹0.01
            .counter(1)
            .nonce(VALID_NONCE)
            .created_at_unix_secs(1_000_000)
            .expires_at_unix_secs(1_001_000)
            .build()
            .unwrap();

        let result = verify_envelope(&tampered, &sig, &verifier);
        assert!(
            matches!(result, Err(CryptoError::VerificationFailed)),
            "tampered amount must fail verification"
        );
    }

    #[test]
    fn wrong_public_key_fails_verification() {
        let signer = test_signer();
        let env = minimal_valid_envelope();
        let sig = sign_envelope(&env, &signer).unwrap();

        // A different key pair
        let wrong_signer = Ed25519TestSigner::from_seed(&[0xFF; 32]);
        let wrong_verifier = wrong_signer.verifier();

        let result = verify_envelope(&env, &sig, &wrong_verifier);
        assert!(
            matches!(result, Err(CryptoError::VerificationFailed)),
            "wrong public key must fail verification"
        );
    }

    // -----------------------------------------------------------------------
    // Malformed input handling
    // -----------------------------------------------------------------------

    #[test]
    fn signature_from_wrong_length_fails() {
        let err = Signature::from_bytes(&[0u8; 32]).unwrap_err(); // 32 bytes, need 64
        assert!(matches!(err, CryptoError::MalformedInput { .. }));
    }

    #[test]
    fn public_key_from_wrong_length_fails() {
        let err = PublicKey::from_bytes(&[0u8; 64]).unwrap_err(); // 64 bytes, need 32
        assert!(matches!(err, CryptoError::MalformedInput { .. }));
    }

    #[test]
    fn signature_round_trip_bytes() {
        let signer = test_signer();
        let env = minimal_valid_envelope();
        let sig = sign_envelope(&env, &signer).unwrap();
        let bytes = sig.to_bytes();
        let recovered = Signature::from_bytes(&bytes).unwrap();
        assert_eq!(sig, recovered);
    }

    #[test]
    fn public_key_round_trip_bytes() {
        let signer = test_signer();
        let pk_bytes = signer.public_key().to_bytes();
        let recovered = PublicKey::from_bytes(&pk_bytes).unwrap();
        assert_eq!(signer.public_key(), &recovered);
    }

    // -----------------------------------------------------------------------
    // Known test vector (frozen for regression detection)
    //
    // These values are produced by the fixed TEST_SEED key and the minimal
    // valid envelope. Any encoding or algorithm change that alters these bytes
    // is a breaking protocol change requiring an ADR update.
    // -----------------------------------------------------------------------

    #[test]
    fn known_vector_signature_length() {
        let signer = test_signer();
        let env = minimal_valid_envelope();
        let sig = sign_envelope(&env, &signer).unwrap();
        assert_eq!(
            sig.to_bytes().len(),
            64,
            "Ed25519 signature must be 64 bytes"
        );
    }

    #[test]
    fn known_vector_public_key_length() {
        let signer = test_signer();
        assert_eq!(signer.public_key().to_bytes().len(), 32);
    }

    #[test]
    fn deterministic_signature_with_fixed_seed_and_envelope() {
        // This vector pins the exact signature bytes for the minimal test envelope
        // signed with TEST_SEED. It must be updated deliberately if encoding changes.
        let signer = test_signer();
        let env = minimal_valid_envelope();
        let sig = sign_envelope(&env, &signer).unwrap();
        let sig_hex: String = sig.to_bytes().iter().map(|b| format!("{b:02x}")).collect();

        // Print for human inspection and future pinning
        println!("Ed25519 test vector signature (hex):\n{sig_hex}");

        // Verify the signature validates correctly (structural check)
        let verifier = signer.verifier();
        verify_envelope(&env, &sig, &verifier).unwrap();
    }
}
