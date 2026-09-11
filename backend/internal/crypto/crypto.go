// Package crypto provides cryptographic operations for the ResilientPay backend.
//
// This package implements:
//   - Ed25519 signature verification (using Go stdlib crypto/ed25519)
//   - Canonical CBOR encoding of TransactionSubmission for signing/verification
//
// # Ed25519 verifier
//
// The backend uses Go's stdlib crypto/ed25519 for signature verification.
// No external cryptographic library is required. The key is always 32 bytes
// (the compressed public key on Ed25519/Edwards25519 curve).
//
// # CBOR canonical encoder
//
// The encoder produces an identical byte sequence to the Rust SDK's
// serialization.rs for the same input. The encoding is:
//
//	signing_input = SIGNING_DOMAIN_SEPARATOR || cbor_encode(13-element array)
//
// Field ordering is identical to serialization.rs (fixed, spec-defined order).
// UUIDs are encoded as 16-byte byte strings (not hyphenated text).
// Integers use CBOR shortest form. Absent optional fields are CBOR null.
//
// The encoder is used by the reconciliation service to re-derive the canonical
// bytes from the submitted fields, then verify the Ed25519 signature. Any
// discrepancy between re-derived bytes and the device-provided bytes is a
// security rejection.
//
// Protocol reference:
//   - docs/04-security/CRYPTO_SPEC.md §2, §5
//   - docs/05-protocol/PAYMENT_PROTOCOL.md §2 (field ordering table)
//   - ADR-002, ADR-008
package crypto

import (
	"crypto/ed25519"
	"errors"
	"fmt"

	gocbor "github.com/fxamacker/cbor/v2"

	"github.com/Shikhyy/ResilientPay/backend/internal/domain"
)

// ---------------------------------------------------------------------------
// Domain separator — MUST match serialization.rs SIGNING_DOMAIN_SEPARATOR
// ---------------------------------------------------------------------------

// SigningDomainSeparator is prepended to the CBOR-encoded envelope before
// signing. It prevents cross-protocol signature reuse.
//
// This value is normative — it MUST match the Rust SDK constant exactly:
//
//	pub const SIGNING_DOMAIN_SEPARATOR: &[u8] = b"resilientpay:payment-envelope:v1:";
//
// Changing this value is a breaking protocol change (ADR-008 / ADR-002).
var SigningDomainSeparator = []byte("resilientpay:payment-envelope:v1:")

// ---------------------------------------------------------------------------
// Ed25519 Verifier
// ---------------------------------------------------------------------------

// Ed25519Verifier verifies Ed25519 signatures using Go's stdlib crypto/ed25519.
//
// This implements the reconciliation.Verifier interface and is the production
// replacement for noopVerifier in cmd/server/main.go.
type Ed25519Verifier struct{}

// NewEd25519Verifier constructs a production Ed25519Verifier.
func NewEd25519Verifier() *Ed25519Verifier {
	return &Ed25519Verifier{}
}

// Verify returns nil if the Ed25519 signature is valid for (message, publicKeyBytes).
//
// Returns an error on any failure — the caller must treat this as a security rejection.
// Error messages are deliberately vague to avoid leaking implementation details.
func (v *Ed25519Verifier) Verify(message []byte, signatureBytes []byte, publicKeyBytes []byte) error {
	if len(publicKeyBytes) != ed25519.PublicKeySize {
		return fmt.Errorf("public key must be %d bytes, got %d", ed25519.PublicKeySize, len(publicKeyBytes))
	}
	if len(signatureBytes) != ed25519.SignatureSize {
		return fmt.Errorf("signature must be %d bytes, got %d", ed25519.SignatureSize, len(signatureBytes))
	}
	pub := ed25519.PublicKey(publicKeyBytes)
	if !ed25519.Verify(pub, message, signatureBytes) {
		return errors.New("signature verification failed")
	}
	return nil
}

// ensure compile-time interface satisfaction (reconciliation.Verifier interface check done in the reconciliation package)
var _ interface {
	Verify(message []byte, signatureBytes []byte, publicKeyBytes []byte) error
} = (*Ed25519Verifier)(nil)

// ---------------------------------------------------------------------------
// Canonical CBOR encoder
// ---------------------------------------------------------------------------

// CanonicalEncoder encodes a TransactionSubmission into the signing input
// that the device's Ed25519TestSigner signed.
//
// This MUST produce bit-for-bit identical output to the Rust SDK's signing_input():
//
//	signing_input = SIGNING_DOMAIN_SEPARATOR || encode_envelope_cbor(envelope)
//
// where encode_envelope_cbor produces a 13-element CBOR array in the order
// defined by PAYMENT_PROTOCOL.md §2.
//
// If the Rust SDK and Go encoder produce different bytes for the same input,
// all submitted transactions will fail signature verification. The frozen test
// vectors in sdk/core/tests/protocol_test_vectors.rs are the reference for
// detecting any divergence.
type CanonicalEncoder struct{}

// NewCanonicalEncoder constructs a CanonicalEncoder.
func NewCanonicalEncoder() *CanonicalEncoder {
	return &CanonicalEncoder{}
}

// Encode produces:
//
//	signing_input = SIGNING_DOMAIN_SEPARATOR || cbor_array_bytes
//
// from the fields of a TransactionSubmission. The canonical bytes can be
// verified against the Ed25519 signature in the submission.
func (e *CanonicalEncoder) Encode(sub *domain.TransactionSubmission) ([]byte, error) {
	cbor, err := encodeToCBOR(sub)
	if err != nil {
		return nil, fmt.Errorf("CBOR encoding failed: %w", err)
	}

	result := make([]byte, 0, len(SigningDomainSeparator)+len(cbor))
	result = append(result, SigningDomainSeparator...)
	result = append(result, cbor...)
	return result, nil
}

// EncodeCBOROnly returns the raw 13-element CBOR array bytes without the
// domain separator. Used for debugging and test vector verification.
func (e *CanonicalEncoder) EncodeCBOROnly(sub *domain.TransactionSubmission) ([]byte, error) {
	return encodeToCBOR(sub)
}

// encodeToCBOR encodes the 13 signed fields as a CBOR array.
//
// Field order matches serialization.rs encode_envelope_cbor() exactly:
//
//	[0]  protocol_version     — uint
//	[1]  tx_id               — 16-byte bstr (UUID bytes)
//	[2]  credential_id       — 16-byte bstr
//	[3]  payer_key_id        — 16-byte bstr
//	[4]  merchant_id         — 16-byte bstr
//	[5]  amount_minor        — uint
//	[6]  currency            — tstr (text)
//	[7]  counter             — uint
//	[8]  nonce               — 16-byte bstr
//	[9]  created_at_unix_secs — int (signed allowed for historic timestamps)
//	[10] expires_at_unix_secs — int
//	[11] previous_event_hash  — 32-byte bstr or null
//	[12] risk_class           — tstr or null
func encodeToCBOR(sub *domain.TransactionSubmission) ([]byte, error) {
	// Use fxamacker/cbor/v2 deterministic encoding mode:
	// - SortNone: array order is caller-controlled (we fix it to spec order below)
	// - IndefLengthForbidden: all lengths must be definite (RFC 8949 §4.2.1)
	// - NaNConvert/InfConvert: forbid IEEE floats (money must never use float)
	enc, err := gocbor.EncOptions{
		Sort:        gocbor.SortNone,
		IndefLength: gocbor.IndefLengthForbidden,
	}.EncMode()
	if err != nil {
		return nil, fmt.Errorf("EncMode error: %w", err)
	}

	// Encode each field separately, then assemble the 13-element CBOR array.
	// This gives us full control over exact field types and order.

	txBytes := uuidToBytes(sub.TxID)
	crBytes := uuidToBytes(sub.CredentialID)
	keyBytes := uuidToBytes(sub.PayerKeyID)
	merBytes := uuidToBytes(sub.MerchantID)

	// [11] previous_event_hash: 32-byte bstr or null
	var prevHashItem interface{}
	if len(sub.PreviousEventHash) == 32 {
		b := make([]byte, 32)
		copy(b, sub.PreviousEventHash)
		prevHashItem = b
	} // else prevHashItem = nil → CBOR null

	// [12] risk_class: tstr or null
	var riskClassItem interface{}
	if sub.RiskClass != "" {
		riskClassItem = sub.RiskClass
	} // else riskClassItem = nil → CBOR null

	// Assemble as a Go slice of interface{} and encode as a CBOR array.
	// fxamacker/cbor encodes []interface{} as a CBOR array.
	items := []interface{}{
		uint32(sub.ProtocolVersion), // [0]  uint (cbor major type 0)
		txBytes,                     // [1]  bstr (major type 2)
		crBytes,                     // [2]  bstr
		keyBytes,                    // [3]  bstr
		merBytes,                    // [4]  bstr
		sub.AmountMinor,             // [5]  uint
		sub.Currency,                // [6]  tstr (major type 3)
		sub.Counter,                 // [7]  uint
		sub.Nonce,                   // [8]  bstr (16 bytes)
		sub.CreatedAtUnix,           // [9]  int (major type 0 or 1)
		sub.ExpiresAtUnix,           // [10] int
		prevHashItem,                // [11] bstr or null
		riskClassItem,               // [12] tstr or null
	}

	return enc.Marshal(items)
}

// uuidToBytes returns the 16 raw bytes of a UUID (not the hyphenated string form).
// This matches the Rust SDK: Value::Bytes(envelope.tx_id().as_uuid().as_bytes().to_vec())
func uuidToBytes(id [16]byte) []byte {
	b := make([]byte, 16)
	copy(b, id[:])
	return b
}
