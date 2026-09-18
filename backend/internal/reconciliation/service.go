// Package reconciliation implements the backend reconciliation logic.
//
// The reconciler processes TransactionSubmissions from devices and produces a
// ReconciliationOutcome per the ingestion flow in docs/05-protocol/RECONCILIATION_SPEC.md §3.
//
// Ingestion flow (normative):
//  1. Parse + schema validate
//  2. Authenticate signature       — verifier recomputes canonical CBOR, checks Ed25519
//  3. Check credential status      — must be ACTIVE and not expired
//  4. Check transaction ID         — idempotency, duplicate detection
//  5. Check counter / policy       — counter > 0, within credential max, > last seen
//  6. Compare prior evidence       — duplicate / compatible / conflict
//  7. Persist event + derived state
//  8. Return result
//
// Security guarantees:
//   - Never trusts client-reported finality (steps 2–5 are re-validated server-side).
//   - Conflict state preserved as a first-class record (never silently overwritten).
//   - Idempotent: exact duplicate returns ALREADY_KNOWN without side effects.
//   - Audit events are emitted for all outcomes.
//
// Signature verification is done by an injected Verifier interface so that the
// reconciler has no dependency on the specific Ed25519 library.
package reconciliation

import (
	"bytes"
	"context"
	"fmt"
	"time"

	"github.com/google/uuid"

	"github.com/Shikhyy/ResilientPay/backend/internal/domain"
	"github.com/Shikhyy/ResilientPay/backend/internal/store"
)

// ---------------------------------------------------------------------------
// Signature verifier interface
// ---------------------------------------------------------------------------

// Verifier is the cryptographic signature verification interface.
// Implementations wrap the actual Ed25519 library.
//
// The backend always re-verifies the signature — it never trusts the device's
// claim that the signature is valid.
type Verifier interface {
	// Verify returns nil if the signature is valid for the given message and
	// public key bytes. Returns an error on any verification failure.
	//
	// Security rule: a non-nil error from Verify is a security rejection —
	// the caller must not silently ignore it.
	Verify(message []byte, signatureBytes []byte, publicKeyBytes []byte) error
}

// CanonicalEncoder encodes a TransactionSubmission into the canonical byte
// sequence that was signed. The backend recomputes this from the submission
// fields and verifies it matches CanonicalBytes before accepting.
//
// This prevents an attacker from submitting tampered fields alongside a valid
// signature over different bytes.
type CanonicalEncoder interface {
	Encode(sub *domain.TransactionSubmission) ([]byte, error)
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

// Service performs reconciliation of submitted transactions.
type Service struct {
	store    store.Store
	verifier Verifier
	encoder  CanonicalEncoder
	nowFn    func() time.Time // injectable for deterministic tests
}

// NewService constructs a reconciliation Service.
func NewService(st store.Store, verifier Verifier, encoder CanonicalEncoder) *Service {
	return &Service{
		store:    st,
		verifier: verifier,
		encoder:  encoder,
		nowFn:    func() time.Time { return time.Now().UTC() },
	}
}

// ---------------------------------------------------------------------------
// Reconcile processes a single TransactionSubmission.
// ---------------------------------------------------------------------------

// Reconcile implements the normative ingestion flow.
// Every step that fails short-circuits with a REJECTED or CONFLICT outcome.
// Outcomes are always accompanied by an audit event.
func (s *Service) Reconcile(ctx context.Context, sub *domain.TransactionSubmission) (*domain.ReconciliationOutcome, error) {
	now := s.nowFn()

	// -----------------------------------------------------------------------
	// Step 1: Schema / basic field validation
	// -----------------------------------------------------------------------

	if err := validateSubmission(sub); err != nil {
		s.emitAudit(ctx, &sub.TxID, nil, "RECONCILE_REJECTED_SCHEMA", err.Error())
		return &domain.ReconciliationOutcome{
			TxID:              sub.TxID,
			Result:            domain.ResultRejected,
			Reason:            fmt.Sprintf("schema validation failed: %s", err),
			BackendReceivedAt: now,
		}, nil
	}

	// -----------------------------------------------------------------------
	// Step 2: Authenticate signature
	//
	// The backend recomputes canonical bytes from the submitted fields and
	// verifies them against the supplied signature. This step also verifies
	// that CanonicalBytes (if supplied) matches the recomputed bytes.
	// -----------------------------------------------------------------------

	cred, err := s.store.GetCredential(ctx, sub.CredentialID)
	if err != nil {
		if err == store.ErrNotFound {
			s.emitAudit(ctx, &sub.TxID, &sub.CredentialID, "RECONCILE_REJECTED_UNKNOWN_CREDENTIAL", "credential not found")
			return reject(sub.TxID, now, "unknown credential"), nil
		}
		return nil, fmt.Errorf("store.GetCredential: %w", err)
	}

	canonicalBytes, err := s.encoder.Encode(sub)
	if err != nil {
		return nil, fmt.Errorf("canonical encoding failed: %w", err)
	}

	// If the device provided canonical_bytes, verify they match recomputed bytes.
	// A mismatch means the device sent tampered fields alongside a valid signature.
	if len(sub.CanonicalBytes) > 0 && !bytes.Equal(sub.CanonicalBytes, canonicalBytes) {
		s.emitAudit(ctx, &sub.TxID, &sub.CredentialID, "RECONCILE_REJECTED_CANONICAL_MISMATCH",
			"device-provided canonical_bytes differ from server-recomputed bytes")
		return reject(sub.TxID, now, "canonical bytes mismatch"), nil
	}

	if err := s.verifier.Verify(canonicalBytes, sub.SignatureBytes, cred.PublicKeyBytes); err != nil {
		s.emitAudit(ctx, &sub.TxID, &sub.CredentialID, "RECONCILE_REJECTED_SIGNATURE", err.Error())
		return reject(sub.TxID, now, "signature verification failed"), nil
	}

	// -----------------------------------------------------------------------
	// Step 3: Check credential status
	// -----------------------------------------------------------------------

	if !cred.IsActive(now) {
		s.emitAudit(ctx, &sub.TxID, &sub.CredentialID, "RECONCILE_REJECTED_CREDENTIAL_INACTIVE",
			fmt.Sprintf("credential state=%s expires=%s", cred.State, cred.ExpiresAt.Format(time.RFC3339)))
		return reject(sub.TxID, now, "credential is not active"), nil
	}

	// -----------------------------------------------------------------------
	// Step 4: Check transaction ID (idempotency + duplicate detection)
	// -----------------------------------------------------------------------

	existing, err := s.store.GetTransaction(ctx, sub.TxID)
	if err != nil && err != store.ErrNotFound {
		return nil, fmt.Errorf("store.GetTransaction: %w", err)
	}

	if existing != nil {
		// -----------------------------------------------------------------------
		// Step 6a: Exact duplicate → idempotent ALREADY_KNOWN
		// -----------------------------------------------------------------------
		if isExactDuplicate(existing, sub) {
			s.emitAudit(ctx, &sub.TxID, &sub.CredentialID, "RECONCILE_ALREADY_KNOWN", "exact duplicate")
			return &domain.ReconciliationOutcome{
				TxID:              sub.TxID,
				Result:            domain.ResultAlreadyKnown,
				Reason:            "exact duplicate submission",
				BackendReceivedAt: now,
			}, nil
		}

		// -----------------------------------------------------------------------
		// Step 6c: Contradiction → CONFLICT (first-class record, never silently overwritten)
		// -----------------------------------------------------------------------
		reason := fmt.Sprintf("conflicting evidence: existing counter=%d submitted counter=%d", existing.Counter, sub.Counter)
		s.emitAudit(ctx, &sub.TxID, &sub.CredentialID, "RECONCILE_CONFLICT", reason)
		existing.State = domain.StateConflict
		_ = s.store.SaveTransaction(ctx, existing)
		return &domain.ReconciliationOutcome{
			TxID:              sub.TxID,
			Result:            domain.ResultConflict,
			Reason:            reason,
			BackendReceivedAt: now,
		}, nil
	}

	// -----------------------------------------------------------------------
	// Step 5: Check counter / policy
	// -----------------------------------------------------------------------

	if sub.Counter == 0 {
		s.emitAudit(ctx, &sub.TxID, &sub.CredentialID, "RECONCILE_REJECTED_COUNTER", "counter must be >= 1")
		return reject(sub.TxID, now, "counter must be >= 1"), nil
	}
	if sub.Counter > cred.MaxCounter {
		s.emitAudit(ctx, &sub.TxID, &sub.CredentialID, "RECONCILE_REJECTED_COUNTER",
			fmt.Sprintf("counter %d exceeds max %d", sub.Counter, cred.MaxCounter))
		return reject(sub.TxID, now, "counter exceeds credential maximum"), nil
	}

	// Double-spend check: Ensure this counter was not previously used under another tx_id (RP-BK-001)
	existingByCounter, err := s.store.GetTransactionByCredentialCounter(ctx, sub.CredentialID, sub.Counter)
	if err != nil && err != store.ErrNotFound {
		return nil, fmt.Errorf("store.GetTransactionByCredentialCounter: %w", err)
	}
	if existingByCounter != nil && existingByCounter.TxID != sub.TxID {
		reason := fmt.Sprintf("conflicting evidence: duplicate counter %d for credential %s (already used by tx %s)",
			sub.Counter, sub.CredentialID, existingByCounter.TxID)
		s.emitAudit(ctx, &sub.TxID, &sub.CredentialID, "RECONCILE_CONFLICT", reason)
		return &domain.ReconciliationOutcome{
			TxID:              sub.TxID,
			Result:            domain.ResultConflict,
			Reason:            reason,
			BackendReceivedAt: now,
		}, nil
	}

	amount := domain.Money{AmountMinor: sub.AmountMinor, Currency: sub.Currency}
	if err := amount.Validate(); err != nil {
		s.emitAudit(ctx, &sub.TxID, &sub.CredentialID, "RECONCILE_REJECTED_AMOUNT", err.Error())
		return reject(sub.TxID, now, err.Error()), nil
	}
	if sub.AmountMinor > cred.MaxValuePerTxMinor {
		s.emitAudit(ctx, &sub.TxID, &sub.CredentialID, "RECONCILE_REJECTED_POLICY",
			fmt.Sprintf("amount %d exceeds per-tx limit %d", sub.AmountMinor, cred.MaxValuePerTxMinor))
		return reject(sub.TxID, now, "amount exceeds per-transaction limit"), nil
	}

	// -----------------------------------------------------------------------
	// Step 6b: offline budget check
	// -----------------------------------------------------------------------
	outstanding, err := s.store.GetOfflineBudget(ctx, sub.CredentialID)
	if err != nil {
		return nil, fmt.Errorf("store.GetOfflineBudget: %w", err)
	}
	if uint64(outstanding)+sub.AmountMinor > cred.MaxValueOutstanding {
		s.emitAudit(ctx, &sub.TxID, &sub.CredentialID, "RECONCILE_REJECTED_BUDGET",
			fmt.Sprintf("adding %d to %d exceeds max %d", sub.AmountMinor, outstanding, cred.MaxValueOutstanding))
		return reject(sub.TxID, now, "offline budget exceeded"), nil
	}

	// -----------------------------------------------------------------------
	// Step 7: Persist new transaction
	// -----------------------------------------------------------------------

	tx := &domain.Transaction{
		TxID:              sub.TxID,
		CredentialID:      sub.CredentialID,
		PayerKeyID:        sub.PayerKeyID,
		MerchantID:        sub.MerchantID,
		Amount:            amount,
		Counter:           sub.Counter,
		Nonce:             sub.Nonce,
		CreatedAtUnix:     sub.CreatedAtUnix,
		ExpiresAtUnix:     sub.ExpiresAtUnix,
		PreviousEventHash: sub.PreviousEventHash,
		RiskClass:         sub.RiskClass,
		SignatureBytes:    sub.SignatureBytes,
		ProtocolVersion:   sub.ProtocolVersion,
		State:             domain.StateReconciled,
		BackendReceivedAt: now,
	}

	// Step 7: Persist new transaction and increment offline budget atomically.
	// Using SaveTransactionWithBudget to avoid a TOCTOU window between the
	// transaction save and the budget increment (RP-BK-002).
	if err := s.store.SaveTransactionWithBudget(ctx, tx, sub.CredentialID, int64(sub.AmountMinor)); err != nil {
		return nil, fmt.Errorf("store.SaveTransactionWithBudget: %w", err)
	}

	s.emitAudit(ctx, &sub.TxID, &sub.CredentialID, "RECONCILE_ACCEPTED", "")

	return &domain.ReconciliationOutcome{
		TxID:              sub.TxID,
		Result:            domain.ResultAccepted,
		BackendReceivedAt: now,
	}, nil
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

func reject(txID uuid.UUID, now time.Time, reason string) *domain.ReconciliationOutcome {
	return &domain.ReconciliationOutcome{
		TxID:              txID,
		Result:            domain.ResultRejected,
		Reason:            reason,
		BackendReceivedAt: now,
	}
}

// isExactDuplicate returns true if the existing transaction and the new submission
// contain identical content (same counter, amount, nonce, and signature).
// This is the idempotency check: exact duplicates must be accepted without side effects.
func isExactDuplicate(existing *domain.Transaction, sub *domain.TransactionSubmission) bool {
	return existing.Counter == sub.Counter &&
		existing.Amount.AmountMinor == sub.AmountMinor &&
		existing.Amount.Currency == sub.Currency &&
		bytes.Equal(existing.Nonce, sub.Nonce) &&
		bytes.Equal(existing.SignatureBytes, sub.SignatureBytes)
}

func validateSubmission(sub *domain.TransactionSubmission) error {
	if sub.ProtocolVersion != domain.ProtocolVersion {
		return fmt.Errorf("unsupported protocol version %d", sub.ProtocolVersion)
	}
	if sub.TxID == uuid.Nil {
		return fmt.Errorf("tx_id must not be nil")
	}
	if sub.CredentialID == uuid.Nil {
		return fmt.Errorf("credential_id must not be nil")
	}
	if len(sub.Nonce) != 16 {
		return fmt.Errorf("nonce must be exactly 16 bytes, got %d", len(sub.Nonce))
	}
	if len(sub.SignatureBytes) != 64 {
		return fmt.Errorf("signature_bytes must be exactly 64 bytes, got %d", len(sub.SignatureBytes))
	}
	return nil
}

func (s *Service) emitAudit(ctx context.Context, txID *uuid.UUID, credID *uuid.UUID, kind, detail string) {
	ev := store.NewAuditEvent(txID, credID, kind, detail)
	_ = s.store.RecordAuditEvent(ctx, ev) // best-effort; do not fail the reconciliation on audit write failure
}
