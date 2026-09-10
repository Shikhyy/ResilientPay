// Package store defines the storage interface and an in-memory implementation
// for the ResilientPay backend.
//
// The interface is designed so that a PostgreSQL implementation can replace
// the in-memory implementation without changing the reconciliation or API layers.
//
// Transaction rules:
//   - SubmitTransaction and RecordOutcome must be idempotent for repeated submissions.
//   - Conflicts must be preserved — never silently overwritten.
//   - All store methods accept a context for cancellation and deadline propagation.
package store

import (
	"context"
	"errors"
	"sync"
	"time"

	"github.com/google/uuid"

	"github.com/Shikhyy/ResilientPay/backend/internal/domain"
)

// ErrNotFound is returned when a requested entity does not exist.
var ErrNotFound = errors.New("not found")

// ErrDuplicate is returned when an entity already exists with the same key.
var ErrDuplicate = errors.New("duplicate")

// ---------------------------------------------------------------------------
// Storage interface
// ---------------------------------------------------------------------------

// Store is the storage contract for the backend.
// All implementations must be safe for concurrent use.
type Store interface {
	// Credential operations
	GetCredential(ctx context.Context, id uuid.UUID) (*domain.Credential, error)
	UpsertCredential(ctx context.Context, c *domain.Credential) error

	// Transaction operations
	GetTransaction(ctx context.Context, txID uuid.UUID) (*domain.Transaction, error)
	// SaveTransaction creates or updates a transaction record.
	// Implementations must ensure idempotency: the same txID submitted twice
	// with identical content must not create a duplicate row.
	SaveTransaction(ctx context.Context, tx *domain.Transaction) error

	// Audit
	RecordAuditEvent(ctx context.Context, ev *domain.AuditEvent) error
	ListAuditEvents(ctx context.Context, txID *uuid.UUID) ([]*domain.AuditEvent, error)
}

// ---------------------------------------------------------------------------
// In-memory implementation (for testing and local development)
// ---------------------------------------------------------------------------

// MemStore is a thread-safe in-memory Store.
// It does NOT survive process restarts and MUST NOT be used in production.
type MemStore struct {
	mu           sync.RWMutex
	credentials  map[uuid.UUID]*domain.Credential
	transactions map[uuid.UUID]*domain.Transaction
	auditEvents  []*domain.AuditEvent
}

// NewMemStore creates an empty in-memory store.
func NewMemStore() *MemStore {
	return &MemStore{
		credentials:  make(map[uuid.UUID]*domain.Credential),
		transactions: make(map[uuid.UUID]*domain.Transaction),
	}
}

func (s *MemStore) GetCredential(ctx context.Context, id uuid.UUID) (*domain.Credential, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	c, ok := s.credentials[id]
	if !ok {
		return nil, ErrNotFound
	}
	cp := *c
	return &cp, nil
}

func (s *MemStore) UpsertCredential(ctx context.Context, c *domain.Credential) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	cp := *c
	s.credentials[c.CredentialID] = &cp
	return nil
}

func (s *MemStore) GetTransaction(ctx context.Context, txID uuid.UUID) (*domain.Transaction, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	tx, ok := s.transactions[txID]
	if !ok {
		return nil, ErrNotFound
	}
	cp := *tx
	return &cp, nil
}

func (s *MemStore) SaveTransaction(ctx context.Context, tx *domain.Transaction) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	cp := *tx
	s.transactions[tx.TxID] = &cp
	return nil
}

func (s *MemStore) RecordAuditEvent(ctx context.Context, ev *domain.AuditEvent) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	cp := *ev
	s.auditEvents = append(s.auditEvents, &cp)
	return nil
}

func (s *MemStore) ListAuditEvents(ctx context.Context, txID *uuid.UUID) ([]*domain.AuditEvent, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	var result []*domain.AuditEvent
	for _, ev := range s.auditEvents {
		if txID == nil || (ev.TxID != nil && *ev.TxID == *txID) {
			cp := *ev
			result = append(result, &cp)
		}
	}
	return result, nil
}

// Snapshot returns a snapshot of all transactions (for testing and inspection).
func (s *MemStore) Snapshot() map[uuid.UUID]domain.Transaction {
	s.mu.RLock()
	defer s.mu.RUnlock()
	out := make(map[uuid.UUID]domain.Transaction, len(s.transactions))
	for k, v := range s.transactions {
		out[k] = *v
	}
	return out
}

// CredentialCount returns the number of stored credentials (for testing).
func (s *MemStore) CredentialCount() int {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return len(s.credentials)
}

// ensure MemStore implements Store at compile time
var _ Store = (*MemStore)(nil)

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

// NewAuditEvent creates an AuditEvent with a generated ID and current timestamp.
func NewAuditEvent(txID *uuid.UUID, credID *uuid.UUID, kind, detail string) *domain.AuditEvent {
	return &domain.AuditEvent{
		EventID:      uuid.New(),
		TxID:         txID,
		CredentialID: credID,
		Kind:         kind,
		Detail:       detail,
		OccurredAt:   time.Now().UTC(),
	}
}
