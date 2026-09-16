// Crypto package tests.
//
// The most important test here is TestCrossSDKTestVector_CBORMatches, which verifies
// that the Go CanonicalEncoder produces byte-for-bit identical output to the Rust SDK
// for the same frozen test input. The expected bytes are the Gate 3 frozen values from:
//
//	sdk/core/tests/protocol_test_vectors.rs (EXPECTED_CBOR_HEX constant)
//
// If this test fails, the Go encoder and Rust encoder have diverged — any new
// transaction submitted to the backend will fail signature verification.
package crypto

import (
	"crypto/ed25519"
	"crypto/rand"
	"encoding/hex"
	"testing"

	"github.com/google/uuid"

	"github.com/Shikhyy/ResilientPay/backend/internal/domain"
)

// ---------------------------------------------------------------------------
// Cross-SDK test vector (Gate 3 freeze)
//
// These values MUST match the frozen constants in:
//   sdk/core/tests/protocol_test_vectors.rs
//
// Inputs:
//   TX UUID  : 00000000-0000-0000-0000-000000000001
//   CR UUID  : 00000000-0000-0000-0000-000000000002
//   Key UUID : 00000000-0000-0000-0000-000000000003
//   Mer UUID : 00000000-0000-0000-0000-000000000004
//   Nonce    : [0x01; 16]
//   Amount   : 150 paise
//   Currency : INR
//   Counter  : 1
//   Created  : 1700000000
//   Expires  : 1700003600
//   Protocol : 1
//
// Expected CBOR (106 bytes): pinned from Rust SDK generate_and_print_vectors
// ---------------------------------------------------------------------------

const crossSDKCBORHex = "8d015000000000000000000000000000000001500000000000000000000000000000000250000000000000000000000000000000035000000000000000000000000000000004189663494e520150010101010101010101010101010101011a6553f1001a6553ff10f6f6"

func mustParseUUID(s string) uuid.UUID {
	u, err := uuid.Parse(s)
	if err != nil {
		panic(err)
	}
	return u
}

func vectorSubmission() *domain.TransactionSubmission {
	return &domain.TransactionSubmission{
		ProtocolVersion:   1,
		TxID:              mustParseUUID("00000000-0000-0000-0000-000000000001"),
		CredentialID:      mustParseUUID("00000000-0000-0000-0000-000000000002"),
		PayerKeyID:        mustParseUUID("00000000-0000-0000-0000-000000000003"),
		MerchantID:        mustParseUUID("00000000-0000-0000-0000-000000000004"),
		AmountMinor:       150,
		Currency:          "INR",
		Counter:           1,
		Nonce:             bytes16(0x01),
		CreatedAtUnix:     1_700_000_000,
		ExpiresAtUnix:     1_700_003_600,
		PreviousEventHash: nil,
		RiskClass:         "",
		SignatureBytes:    make([]byte, 64),
	}
}

func bytes16(b byte) []byte {
	buf := make([]byte, 16)
	for i := range buf {
		buf[i] = b
	}
	return buf
}

// TestCrossSDKTestVector_CBORMatches is the critical cross-language byte equality test.
// A failure here means the Go and Rust CBOR encoders have diverged.
func TestCrossSDKTestVector_CBORMatches(t *testing.T) {
	enc := NewCanonicalEncoder()
	sub := vectorSubmission()

	actualCBOR, err := enc.EncodeCBOROnly(sub)
	if err != nil {
		t.Fatalf("EncodeCBOROnly failed: %v", err)
	}

	expectedCBOR, err := hex.DecodeString(crossSDKCBORHex)
	if err != nil {
		t.Fatalf("bad test hex: %v", err)
	}

	if len(actualCBOR) != len(expectedCBOR) {
		t.Errorf("CBOR length mismatch: expected %d bytes, got %d", len(expectedCBOR), len(actualCBOR))
		t.Errorf("Expected: %s", crossSDKCBORHex)
		t.Errorf("Actual:   %s", hex.EncodeToString(actualCBOR))
		t.FailNow()
	}

	for i := range actualCBOR {
		if actualCBOR[i] != expectedCBOR[i] {
			t.Errorf("CBOR byte mismatch at position %d: expected 0x%02x, got 0x%02x", i, expectedCBOR[i], actualCBOR[i])
		}
	}

	if t.Failed() {
		t.Errorf("\nExpected: %s\nActual:   %s", crossSDKCBORHex, hex.EncodeToString(actualCBOR))
		t.Logf("\nThis test failure means the Go CBOR encoder has diverged from the Rust SDK.")
		t.Logf("Any transactions submitted to the backend will fail signature verification.")
		t.Logf("Review serialization.rs and crypto.go side by side to find the divergence.")
	} else {
		t.Logf("Cross-SDK CBOR vector matches: %d bytes, hex=%s", len(actualCBOR), crossSDKCBORHex)
	}
}

func TestDecodeFromCBOR_MatchesVector(t *testing.T) {
	expectedCBOR, err := hex.DecodeString(crossSDKCBORHex)
	if err != nil {
		t.Fatalf("bad test hex: %v", err)
	}

	decoded, err := DecodeFromCBOR(expectedCBOR)
	if err != nil {
		t.Fatalf("DecodeFromCBOR failed: %v", err)
	}

	original := vectorSubmission()
	if decoded.ProtocolVersion != original.ProtocolVersion {
		t.Errorf("ProtocolVersion mismatch: got %d, want %d", decoded.ProtocolVersion, original.ProtocolVersion)
	}
	if decoded.TxID != original.TxID {
		t.Errorf("TxID mismatch: got %s, want %s", decoded.TxID, original.TxID)
	}
	if decoded.CredentialID != original.CredentialID {
		t.Errorf("CredentialID mismatch: got %s, want %s", decoded.CredentialID, original.CredentialID)
	}
	if decoded.PayerKeyID != original.PayerKeyID {
		t.Errorf("PayerKeyID mismatch: got %s, want %s", decoded.PayerKeyID, original.PayerKeyID)
	}
	if decoded.MerchantID != original.MerchantID {
		t.Errorf("MerchantID mismatch: got %s, want %s", decoded.MerchantID, original.MerchantID)
	}
	if decoded.AmountMinor != original.AmountMinor {
		t.Errorf("AmountMinor mismatch: got %d, want %d", decoded.AmountMinor, original.AmountMinor)
	}
	if decoded.Currency != original.Currency {
		t.Errorf("Currency mismatch: got %s, want %s", decoded.Currency, original.Currency)
	}
	if decoded.Counter != original.Counter {
		t.Errorf("Counter mismatch: got %d, want %d", decoded.Counter, original.Counter)
	}
	if string(decoded.Nonce) != string(original.Nonce) {
		t.Errorf("Nonce mismatch")
	}
	if decoded.CreatedAtUnix != original.CreatedAtUnix {
		t.Errorf("CreatedAtUnix mismatch: got %d, want %d", decoded.CreatedAtUnix, original.CreatedAtUnix)
	}
	if decoded.ExpiresAtUnix != original.ExpiresAtUnix {
		t.Errorf("ExpiresAtUnix mismatch: got %d, want %d", decoded.ExpiresAtUnix, original.ExpiresAtUnix)
	}
}

// TestCrossSDKTestVector_SigningInputMatchesLength verifies the signing input
// (domain separator || CBOR) has the expected total length.
func TestCrossSDKTestVector_SigningInputLength(t *testing.T) {
	enc := NewCanonicalEncoder()
	sub := vectorSubmission()

	signingInput, err := enc.Encode(sub)
	if err != nil {
		t.Fatalf("Encode failed: %v", err)
	}

	// Domain separator = 33 bytes ("resilientpay:payment-envelope:v1:")
	// CBOR = 106 bytes
	// Total = 139 bytes — matches Rust SDK test output
	const expectedLen = 139
	if len(signingInput) != expectedLen {
		t.Errorf("signing input length mismatch: expected %d, got %d", expectedLen, len(signingInput))
	}

	if !startsWith(signingInput, SigningDomainSeparator) {
		t.Error("signing input does not start with domain separator")
	}
}

func startsWith(data, prefix []byte) bool {
	if len(data) < len(prefix) {
		return false
	}
	for i, b := range prefix {
		if data[i] != b {
			return false
		}
	}
	return true
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

func TestCanonicalEncoder_IsDeterministic(t *testing.T) {
	enc := NewCanonicalEncoder()
	sub := vectorSubmission()

	b1, _ := enc.Encode(sub)
	b2, _ := enc.Encode(sub)
	b3, _ := enc.Encode(sub)

	if string(b1) != string(b2) || string(b2) != string(b3) {
		t.Error("canonical encoding is not deterministic")
	}
}

// ---------------------------------------------------------------------------
// Field sensitivity: different inputs → different bytes
// ---------------------------------------------------------------------------

func TestCanonicalEncoder_DifferentAmountsProduceDifferentBytes(t *testing.T) {
	enc := NewCanonicalEncoder()

	s1 := vectorSubmission()
	s1.AmountMinor = 100

	s2 := vectorSubmission()
	s2.AmountMinor = 200

	b1, _ := enc.EncodeCBOROnly(s1)
	b2, _ := enc.EncodeCBOROnly(s2)

	if string(b1) == string(b2) {
		t.Error("different amounts must produce different CBOR bytes")
	}
}

func TestCanonicalEncoder_DifferentCountersProduceDifferentBytes(t *testing.T) {
	enc := NewCanonicalEncoder()

	s1 := vectorSubmission()
	s1.Counter = 1

	s2 := vectorSubmission()
	s2.Counter = 2

	b1, _ := enc.EncodeCBOROnly(s1)
	b2, _ := enc.EncodeCBOROnly(s2)

	if string(b1) == string(b2) {
		t.Error("different counters must produce different CBOR bytes")
	}
}

// ---------------------------------------------------------------------------
// Ed25519 Verifier
// ---------------------------------------------------------------------------

func TestEd25519Verifier_ValidSignatureAccepted(t *testing.T) {
	pub, priv, err := ed25519.GenerateKey(rand.Reader)
	if err != nil {
		t.Fatalf("key generation failed: %v", err)
	}

	message := []byte("resilientpay:payment-envelope:v1:test-message")
	sig := ed25519.Sign(priv, message)

	verifier := NewEd25519Verifier()
	if err := verifier.Verify(message, sig, pub); err != nil {
		t.Errorf("valid signature must be accepted: %v", err)
	}
}

func TestEd25519Verifier_TamperedMessageRejected(t *testing.T) {
	pub, priv, _ := ed25519.GenerateKey(rand.Reader)

	original := []byte("resilientpay:payment-envelope:v1:original-message")
	sig := ed25519.Sign(priv, original)

	tampered := []byte("resilientpay:payment-envelope:v1:tampered-message")
	verifier := NewEd25519Verifier()
	if err := verifier.Verify(tampered, sig, pub); err == nil {
		t.Error("tampered message must fail verification")
	}
}

func TestEd25519Verifier_WrongKeyRejected(t *testing.T) {
	_, priv, _ := ed25519.GenerateKey(rand.Reader)
	wrongPub, _, _ := ed25519.GenerateKey(rand.Reader)

	message := []byte("test message")
	sig := ed25519.Sign(priv, message)

	verifier := NewEd25519Verifier()
	if err := verifier.Verify(message, sig, wrongPub); err == nil {
		t.Error("wrong public key must fail verification")
	}
}

func TestEd25519Verifier_WrongKeyLengthRejected(t *testing.T) {
	verifier := NewEd25519Verifier()
	err := verifier.Verify([]byte("msg"), make([]byte, 64), make([]byte, 16)) // 16 bytes instead of 32
	if err == nil {
		t.Error("wrong key length must fail")
	}
}

func TestEd25519Verifier_WrongSigLengthRejected(t *testing.T) {
	verifier := NewEd25519Verifier()
	err := verifier.Verify([]byte("msg"), make([]byte, 32), make([]byte, 32)) // 32 bytes instead of 64
	if err == nil {
		t.Error("wrong signature length must fail")
	}
}

// ---------------------------------------------------------------------------
// Round-trip: encode → sign → verify (using stdlib ed25519 directly)
// ---------------------------------------------------------------------------

func TestRoundTrip_EncodeSignVerify(t *testing.T) {
	// Generate a key pair using stdlib (deterministic in principle, random here for test)
	pub, priv, err := ed25519.GenerateKey(rand.Reader)
	if err != nil {
		t.Fatalf("key generation: %v", err)
	}

	enc := NewCanonicalEncoder()
	verifier := NewEd25519Verifier()

	sub := vectorSubmission()
	sub.CredentialID = uuid.New() // random credential to make it distinct

	// Encode to signing input
	signingInput, err := enc.Encode(sub)
	if err != nil {
		t.Fatalf("Encode: %v", err)
	}

	// Sign with stdlib Ed25519 (same algorithm as ed25519-dalek in Rust)
	sig := ed25519.Sign(priv, signingInput)
	sub.SignatureBytes = sig

	// Verify using the backend verifier
	// Re-encode to get the canonical bytes (as the backend would do)
	canonicalBytes, _ := enc.Encode(sub)
	if err := verifier.Verify(canonicalBytes, sub.SignatureBytes, pub); err != nil {
		t.Errorf("round-trip verification failed: %v", err)
	}
}
