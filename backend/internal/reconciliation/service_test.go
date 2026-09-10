// Package reconciliation tests — service-level unit tests with in-memory store.
package reconciliation

import (
	"context"
	"testing"
	"time"

	"github.com/google/uuid"

	"github.com/Shikhyy/ResilientPay/backend/internal/domain"
	"github.com/Shikhyy/ResilientPay/backend/internal/store"
)

// ---------------------------------------------------------------------------
// Test doubles
// ---------------------------------------------------------------------------

// alwaysValidVerifier accepts any signature.
type alwaysValidVerifier struct{}

func (alwaysValidVerifier) Verify(_, _, _ []byte) error { return nil }

// alwaysInvalidVerifier rejects every signature.
type alwaysInvalidVerifier struct{}

func (alwaysInvalidVerifier) Verify(_, _, _ []byte) error {
	return domain.ErrInvalidSignature
}

// noopEncoder always returns a fixed canonical byte slice.
type noopEncoder struct{ bytes []byte }

func (e *noopEncoder) Encode(_ *domain.TransactionSubmission) ([]byte, error) {
	if e.bytes == nil {
		return []byte("canonical-placeholder"), nil
	}
	return e.bytes, nil
}

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

func makeCredential(state domain.CredentialState, expiresIn time.Duration) *domain.Credential {
	return &domain.Credential{
		CredentialID:        uuid.New(),
		SubjectKeyID:        uuid.New(),
		PublicKeyBytes:      make([]byte, 32),
		IssuedAt:            time.Now().UTC().Add(-1 * time.Hour),
		ExpiresAt:           time.Now().UTC().Add(expiresIn),
		MaxValuePerTxMinor:  50_000,
		MaxValueOutstanding: 200_000,
		MaxCounter:          1_000,
		State:               state,
		PolicyVersion:       1,
	}
}

func makeSubmission(credID uuid.UUID, counter uint64, amountMinor uint64) *domain.TransactionSubmission {
	return &domain.TransactionSubmission{
		ProtocolVersion: domain.ProtocolVersion,
		TxID:            uuid.New(),
		CredentialID:    credID,
		PayerKeyID:      uuid.New(),
		MerchantID:      uuid.New(),
		AmountMinor:     amountMinor,
		Currency:        "INR",
		Counter:         counter,
		Nonce:           make([]byte, 16),
		CreatedAtUnix:   time.Now().Unix() - 10,
		ExpiresAtUnix:   time.Now().Unix() + 3600,
		SignatureBytes:  make([]byte, 64),
	}
}

func newService(verifier Verifier) (*Service, *store.MemStore) {
	st := store.NewMemStore()
	svc := NewService(st, verifier, &noopEncoder{})
	return svc, st
}

// ---------------------------------------------------------------------------
// Happy path
// ---------------------------------------------------------------------------

func TestReconcile_AcceptsValidSubmission(t *testing.T) {
	svc, st := newService(alwaysValidVerifier{})
	ctx := context.Background()

	cred := makeCredential(domain.CredentialActive, 1*time.Hour)
	_ = st.UpsertCredential(ctx, cred)

	sub := makeSubmission(cred.CredentialID, 1, 10_000)
	out, err := svc.Reconcile(ctx, sub)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if out.Result != domain.ResultAccepted {
		t.Errorf("expected ACCEPTED, got %s: %s", out.Result, out.Reason)
	}
}

// ---------------------------------------------------------------------------
// Idempotency
// ---------------------------------------------------------------------------

func TestReconcile_ExactDuplicateIsAlreadyKnown(t *testing.T) {
	svc, st := newService(alwaysValidVerifier{})
	ctx := context.Background()

	cred := makeCredential(domain.CredentialActive, 1*time.Hour)
	_ = st.UpsertCredential(ctx, cred)

	sub := makeSubmission(cred.CredentialID, 1, 10_000)

	// First submission
	out1, err := svc.Reconcile(ctx, sub)
	if err != nil || out1.Result != domain.ResultAccepted {
		t.Fatalf("first submission failed: %v %v", err, out1)
	}

	// Identical second submission
	out2, err := svc.Reconcile(ctx, sub)
	if err != nil {
		t.Fatalf("unexpected error on second submission: %v", err)
	}
	if out2.Result != domain.ResultAlreadyKnown {
		t.Errorf("expected ALREADY_KNOWN on duplicate, got %s", out2.Result)
	}
}

// ---------------------------------------------------------------------------
// Conflict detection
// ---------------------------------------------------------------------------

func TestReconcile_ConflictingCounterIsConflict(t *testing.T) {
	svc, st := newService(alwaysValidVerifier{})
	ctx := context.Background()

	cred := makeCredential(domain.CredentialActive, 1*time.Hour)
	_ = st.UpsertCredential(ctx, cred)

	// First submission with counter=1 and amount=100
	sub1 := makeSubmission(cred.CredentialID, 1, 100)
	out1, _ := svc.Reconcile(ctx, sub1)
	if out1.Result != domain.ResultAccepted {
		t.Fatalf("first submission expected ACCEPTED: %v", out1)
	}

	// Same tx_id but different counter — conflict
	sub2 := &domain.TransactionSubmission{
		ProtocolVersion: domain.ProtocolVersion,
		TxID:            sub1.TxID, // same tx_id
		CredentialID:    cred.CredentialID,
		PayerKeyID:      uuid.New(),
		MerchantID:      uuid.New(),
		AmountMinor:     999, // different amount
		Currency:        "INR",
		Counter:         2, // different counter
		Nonce:           make([]byte, 16),
		CreatedAtUnix:   time.Now().Unix(),
		ExpiresAtUnix:   time.Now().Unix() + 3600,
		SignatureBytes:  make([]byte, 64),
	}
	out2, err := svc.Reconcile(ctx, sub2)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if out2.Result != domain.ResultConflict {
		t.Errorf("expected CONFLICT, got %s: %s", out2.Result, out2.Reason)
	}
}

// ---------------------------------------------------------------------------
// Security: signature failure
// ---------------------------------------------------------------------------

func TestReconcile_InvalidSignatureIsRejected(t *testing.T) {
	svc, st := newService(alwaysInvalidVerifier{})
	ctx := context.Background()

	cred := makeCredential(domain.CredentialActive, 1*time.Hour)
	_ = st.UpsertCredential(ctx, cred)

	sub := makeSubmission(cred.CredentialID, 1, 10_000)
	out, err := svc.Reconcile(ctx, sub)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if out.Result != domain.ResultRejected {
		t.Errorf("expected REJECTED for invalid signature, got %s", out.Result)
	}
}

// ---------------------------------------------------------------------------
// Credential not active
// ---------------------------------------------------------------------------

func TestReconcile_SuspendedCredentialIsRejected(t *testing.T) {
	svc, st := newService(alwaysValidVerifier{})
	ctx := context.Background()

	cred := makeCredential(domain.CredentialSuspended, 1*time.Hour)
	_ = st.UpsertCredential(ctx, cred)

	sub := makeSubmission(cred.CredentialID, 1, 10_000)
	out, _ := svc.Reconcile(ctx, sub)
	if out.Result != domain.ResultRejected {
		t.Errorf("expected REJECTED for suspended credential, got %s: %s", out.Result, out.Reason)
	}
}

func TestReconcile_ExpiredCredentialIsRejected(t *testing.T) {
	svc, st := newService(alwaysValidVerifier{})
	ctx := context.Background()

	cred := makeCredential(domain.CredentialActive, -1*time.Minute) // already expired
	_ = st.UpsertCredential(ctx, cred)

	sub := makeSubmission(cred.CredentialID, 1, 10_000)
	out, _ := svc.Reconcile(ctx, sub)
	if out.Result != domain.ResultRejected {
		t.Errorf("expected REJECTED for expired credential, got %s: %s", out.Result, out.Reason)
	}
}

// ---------------------------------------------------------------------------
// Counter / policy
// ---------------------------------------------------------------------------

func TestReconcile_ZeroCounterIsRejected(t *testing.T) {
	svc, st := newService(alwaysValidVerifier{})
	ctx := context.Background()

	cred := makeCredential(domain.CredentialActive, 1*time.Hour)
	_ = st.UpsertCredential(ctx, cred)

	sub := makeSubmission(cred.CredentialID, 0, 10_000) // counter = 0 is invalid
	out, _ := svc.Reconcile(ctx, sub)
	if out.Result != domain.ResultRejected {
		t.Errorf("expected REJECTED for counter=0, got %s", out.Result)
	}
}

func TestReconcile_AmountExceedsPerTxLimitIsRejected(t *testing.T) {
	svc, st := newService(alwaysValidVerifier{})
	ctx := context.Background()

	cred := makeCredential(domain.CredentialActive, 1*time.Hour) // MaxValuePerTxMinor = 50_000
	_ = st.UpsertCredential(ctx, cred)

	sub := makeSubmission(cred.CredentialID, 1, 60_000) // exceeds limit
	out, _ := svc.Reconcile(ctx, sub)
	if out.Result != domain.ResultRejected {
		t.Errorf("expected REJECTED for amount exceeding per-tx limit, got %s", out.Result)
	}
}

func TestReconcile_CounterExceedsMaxIsRejected(t *testing.T) {
	svc, st := newService(alwaysValidVerifier{})
	ctx := context.Background()

	cred := makeCredential(domain.CredentialActive, 1*time.Hour) // MaxCounter = 1_000
	_ = st.UpsertCredential(ctx, cred)

	sub := makeSubmission(cred.CredentialID, 1_001, 100) // counter > max
	out, _ := svc.Reconcile(ctx, sub)
	if out.Result != domain.ResultRejected {
		t.Errorf("expected REJECTED for counter > max, got %s", out.Result)
	}
}

// ---------------------------------------------------------------------------
// Schema validation
// ---------------------------------------------------------------------------

func TestReconcile_WrongProtocolVersionIsRejected(t *testing.T) {
	svc, st := newService(alwaysValidVerifier{})
	ctx := context.Background()

	cred := makeCredential(domain.CredentialActive, 1*time.Hour)
	_ = st.UpsertCredential(ctx, cred)

	sub := makeSubmission(cred.CredentialID, 1, 100)
	sub.ProtocolVersion = 99 // wrong version
	out, _ := svc.Reconcile(ctx, sub)
	if out.Result != domain.ResultRejected {
		t.Errorf("expected REJECTED for wrong protocol version, got %s", out.Result)
	}
}

func TestReconcile_NilTxIDIsRejected(t *testing.T) {
	svc, st := newService(alwaysValidVerifier{})
	ctx := context.Background()

	cred := makeCredential(domain.CredentialActive, 1*time.Hour)
	_ = st.UpsertCredential(ctx, cred)

	sub := makeSubmission(cred.CredentialID, 1, 100)
	sub.TxID = uuid.Nil // nil UUID is invalid
	out, _ := svc.Reconcile(ctx, sub)
	if out.Result != domain.ResultRejected {
		t.Errorf("expected REJECTED for nil tx_id, got %s", out.Result)
	}
}

func TestReconcile_UnknownCredentialIsRejected(t *testing.T) {
	svc, _ := newService(alwaysValidVerifier{})
	ctx := context.Background()

	sub := makeSubmission(uuid.New(), 1, 100) // credential not in store
	out, _ := svc.Reconcile(ctx, sub)
	if out.Result != domain.ResultRejected {
		t.Errorf("expected REJECTED for unknown credential, got %s", out.Result)
	}
}
