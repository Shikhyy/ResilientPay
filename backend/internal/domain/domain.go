// Package domain defines the core domain types for the ResilientPay backend.
//
// These types are the authoritative representation of protocol entities as stored
// and processed by the backend. They are NOT shared with the client SDK (which is
// Rust) but mirror the same conceptual model.
//
// Rules:
//   - Money is always integer minor units with an explicit currency string.
//   - Transaction IDs and Credential IDs are UUIDs (16 bytes, canonical hyphenated form in text).
//   - Counters are uint64; zero is invalid in protocol use.
//   - No floating point for any financial field.
//   - Enum types use string constants to make migrations explicit.
//
// Protocol reference:
//   - docs/05-protocol/PAYMENT_PROTOCOL.md
//   - docs/06-development/DATA_MODEL.md
//   - docs/06-development/DATABASE_SCHEMA.md
package domain

import (
	"errors"
	"time"

	"github.com/google/uuid"
)

// ErrInvalidSignature is returned by Verifier implementations when signature
// verification fails. It is a security rejection — callers must not silently ignore it.
var ErrInvalidSignature = errors.New("invalid signature")

// ProtocolVersion is the only protocol version this backend accepts.
// Changing it requires a protocol change-control review and ADR update.
const ProtocolVersion uint32 = 1

// ---------------------------------------------------------------------------
// Money
// ---------------------------------------------------------------------------

// Money represents a monetary amount as an integer number of minor units
// with an explicit currency code.
//
// Examples (INR):
//   - 1 paise = Money{AmountMinor: 1, Currency: "INR"}
//   - ₹1.50   = Money{AmountMinor: 150, Currency: "INR"}
//   - ₹1000   = Money{AmountMinor: 100_000, Currency: "INR"}
//
// Security rule: NEVER use float64 for monetary amounts.
type Money struct {
	AmountMinor uint64 `json:"amount_minor"`
	Currency    string `json:"currency"`
}

// MaxAmountMinor is the ceiling for a single transaction in paise (₹1,000,000).
const MaxAmountMinor uint64 = 100_000_000

// Validate checks that the Money value is well-formed.
func (m Money) Validate() error {
	if len(m.Currency) == 0 || len(m.Currency) > 8 {
		return errors.New("currency code must be 1–8 characters")
	}
	if m.AmountMinor > MaxAmountMinor {
		return errors.New("amount_minor exceeds maximum")
	}
	return nil
}

// ---------------------------------------------------------------------------
// Transaction state
// ---------------------------------------------------------------------------

// TransactionState is the canonical state of a transaction on the backend.
// These values match the normative state names in
// docs/05-protocol/TRANSACTION_STATE_MACHINE.md.
type TransactionState string

const (
	StateCreated         TransactionState = "CREATED"
	StateValidating      TransactionState = "VALIDATING"
	StateAuthorized      TransactionState = "AUTHORIZED"
	StateSigned          TransactionState = "SIGNED"
	StateTransferred     TransactionState = "TRANSFERRED"
	StateReceived        TransactionState = "RECEIVED"
	StateLocallyVerified TransactionState = "LOCALLY_VERIFIED"
	StateLocallyRecorded TransactionState = "LOCALLY_RECORDED"
	StateSyncPending     TransactionState = "SYNC_PENDING"
	StateReconciled      TransactionState = "RECONCILED"
	StateRejected        TransactionState = "REJECTED"
	StateConflict        TransactionState = "CONFLICT"
)

// IsTerminal returns true if no further transitions are permitted.
func (s TransactionState) IsTerminal() bool {
	switch s {
	case StateReconciled, StateRejected, StateConflict:
		return true
	}
	return false
}

// ---------------------------------------------------------------------------
// Credential lifecycle
// ---------------------------------------------------------------------------

// CredentialState is the lifecycle state of an offline credential.
type CredentialState string

const (
	CredentialRequested CredentialState = "REQUESTED"
	CredentialIssued    CredentialState = "ISSUED"
	CredentialActive    CredentialState = "ACTIVE"
	CredentialSuspended CredentialState = "SUSPENDED"
	CredentialRevoked   CredentialState = "REVOKED"
	CredentialExpired   CredentialState = "EXPIRED"
)

// ---------------------------------------------------------------------------
// Reconciliation result codes
// ---------------------------------------------------------------------------

// ReconciliationResult is the outcome of processing a submitted transaction.
type ReconciliationResult string

const (
	// ResultAccepted — first-time valid submission, recorded.
	ResultAccepted ReconciliationResult = "ACCEPTED"
	// ResultAlreadyKnown — exact duplicate of a previously accepted submission.
	ResultAlreadyKnown ReconciliationResult = "ALREADY_KNOWN"
	// ResultRejected — the submission fails validation or policy.
	ResultRejected ReconciliationResult = "REJECTED"
	// ResultConflict — contradictory evidence detected.
	ResultConflict ReconciliationResult = "CONFLICT"
)

// ---------------------------------------------------------------------------
// Core entities
// ---------------------------------------------------------------------------

// Transaction is the backend's authoritative record of a payment transaction.
type Transaction struct {
	TxID              uuid.UUID        `json:"tx_id"`
	CredentialID      uuid.UUID        `json:"credential_id"`
	PayerKeyID        uuid.UUID        `json:"payer_key_id"`
	MerchantID        uuid.UUID        `json:"merchant_id"`
	Amount            Money            `json:"amount"`
	Counter           uint64           `json:"counter"`
	Nonce             []byte           `json:"nonce"`           // 16 bytes
	CreatedAtUnix     int64            `json:"created_at_unix"` // from envelope
	ExpiresAtUnix     int64            `json:"expires_at_unix"`
	PreviousEventHash []byte           `json:"previous_event_hash,omitempty"` // 32 bytes or nil
	RiskClass         string           `json:"risk_class,omitempty"`
	SignatureBytes    []byte           `json:"signature_bytes"` // 64 bytes (Ed25519)
	ProtocolVersion   uint32           `json:"protocol_version"`
	State             TransactionState `json:"state"`
	BackendReceivedAt time.Time        `json:"backend_received_at"`
}

// Credential is the backend's record of an offline authorization credential.
type Credential struct {
	CredentialID        uuid.UUID       `json:"credential_id"`
	SubjectKeyID        uuid.UUID       `json:"subject_key_id"`
	PublicKeyBytes      []byte          `json:"public_key_bytes"` // 32 bytes (Ed25519 verifying key)
	IssuedAt            time.Time       `json:"issued_at"`
	ExpiresAt           time.Time       `json:"expires_at"`
	MaxValuePerTxMinor  uint64          `json:"max_value_per_tx_minor"`
	MaxValueOutstanding uint64          `json:"max_value_outstanding_minor"`
	MaxCounter          uint64          `json:"max_counter"`
	State               CredentialState `json:"state"`
	PolicyVersion       uint32          `json:"policy_version"`
}

// IsActive returns true if the credential is active and not expired at t.
func (c *Credential) IsActive(t time.Time) bool {
	return c.State == CredentialActive && t.Before(c.ExpiresAt)
}

// AuditEvent is an immutable record of a security or reconciliation event.
type AuditEvent struct {
	EventID      uuid.UUID  `json:"event_id"`
	TxID         *uuid.UUID `json:"tx_id,omitempty"`
	CredentialID *uuid.UUID `json:"credential_id,omitempty"`
	Kind         string     `json:"kind"`
	Detail       string     `json:"detail"`
	OccurredAt   time.Time  `json:"occurred_at"`
}

// ---------------------------------------------------------------------------
// Submission (input from device)
// ---------------------------------------------------------------------------

// TransactionSubmission is the payload the device sends for reconciliation.
// It contains the canonical envelope fields plus the signature.
type TransactionSubmission struct {
	ProtocolVersion   uint32    `json:"protocol_version"`
	TxID              uuid.UUID `json:"tx_id"`
	CredentialID      uuid.UUID `json:"credential_id"`
	PayerKeyID        uuid.UUID `json:"payer_key_id"`
	MerchantID        uuid.UUID `json:"merchant_id"`
	AmountMinor       uint64    `json:"amount_minor"`
	Currency          string    `json:"currency"`
	Counter           uint64    `json:"counter"`
	Nonce             []byte    `json:"nonce"`
	CreatedAtUnix     int64     `json:"created_at_unix"`
	ExpiresAtUnix     int64     `json:"expires_at_unix"`
	PreviousEventHash []byte    `json:"previous_event_hash,omitempty"`
	RiskClass         string    `json:"risk_class,omitempty"`
	SignatureBytes    []byte    `json:"signature_bytes"`
	// The canonical CBOR bytes the device signed. The backend MUST recompute
	// these from the fields above and reject submissions where they differ.
	// Including them here allows the backend to detect encoding tampering.
	CanonicalBytes []byte `json:"canonical_bytes"`
}

// ---------------------------------------------------------------------------
// Reconciliation outcome
// ---------------------------------------------------------------------------

// ReconciliationOutcome is returned by the reconciliation service for each
// submitted transaction.
type ReconciliationOutcome struct {
	TxID              uuid.UUID            `json:"tx_id"`
	Result            ReconciliationResult `json:"result"`
	Reason            string               `json:"reason,omitempty"`
	BackendReceivedAt time.Time            `json:"backend_received_at"`
}
