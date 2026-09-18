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
	GetTransactionByCredentialCounter(ctx context.Context, credentialID uuid.UUID, counter uint64) (*domain.Transaction, error)
	// SaveTransaction creates or updates a transaction record.
	// Implementations must ensure idempotency: the same txID submitted twice
	// with identical content must not create a duplicate row.
	SaveTransaction(ctx context.Context, tx *domain.Transaction) error

	// Audit
	RecordAuditEvent(ctx context.Context, ev *domain.AuditEvent) error
	ListAuditEvents(ctx context.Context, txID *uuid.UUID) ([]*domain.AuditEvent, error)

	// Budget operations
	// GetOfflineBudget returns the current outstanding minor-unit balance for a credential.
	// Returns 0 and no error if no record exists yet (first transaction).
	GetOfflineBudget(ctx context.Context, credentialID uuid.UUID) (outstandingMinor int64, err error)

	// UpdateOfflineBudget atomically adds deltaMinor to the outstanding balance.
	// deltaMinor must be positive (adding spend). Negative values are not supported.
	UpdateOfflineBudget(ctx context.Context, credentialID uuid.UUID, deltaMinor int64) error

	// SaveTransactionWithBudget atomically saves a transaction record and increments
	// the offline budget for credentialID by deltaMinor under a single write lock.
	// This eliminates the TOCTOU window between a separate SaveTransaction +
	// UpdateOfflineBudget call pair.
	// deltaMinor must be positive.
	SaveTransactionWithBudget(ctx context.Context, tx *domain.Transaction, credentialID uuid.UUID, deltaMinor int64) error

	// Settlement operations
	GetUnsettledTransactions(ctx context.Context, limit int) ([]*domain.Transaction, error)
	MarkTransactionSettled(ctx context.Context, txID uuid.UUID) error
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
	budgets      map[uuid.UUID]int64
}

// NewMemStore creates an empty in-memory store.
func NewMemStore() *MemStore {
	return &MemStore{
		credentials:  make(map[uuid.UUID]*domain.Credential),
		transactions: make(map[uuid.UUID]*domain.Transaction),
		budgets:      make(map[uuid.UUID]int64),
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

func (s *MemStore) GetTransactionByCredentialCounter(ctx context.Context, credentialID uuid.UUID, counter uint64) (*domain.Transaction, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	for _, tx := range s.transactions {
		if tx.CredentialID == credentialID && tx.Counter == counter {
			cp := *tx
			return &cp, nil
		}
	}
	return nil, ErrNotFound
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


func (s *MemStore) GetUnsettledTransactions(ctx context.Context, limit int) ([]*domain.Transaction, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	var res []*domain.Transaction
	for _, tx := range s.transactions {
		if tx.State == domain.StateReconciled {
			res = append(res, tx)
			if len(res) == limit {
				break
			}
		}
	}
	return res, nil
}

func (s *MemStore) MarkTransactionSettled(ctx context.Context, txID uuid.UUID) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	if tx, ok := s.transactions[txID]; ok {
		if tx.State == domain.StateReconciled {
			tx.State = domain.StateSettled
		}
		return nil
	}
	return ErrNotFound
}

func (s *MemStore) GetOfflineBudget(ctx context.Context, credentialID uuid.UUID) (int64, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.budgets[credentialID], nil
}

func (s *MemStore) UpdateOfflineBudget(ctx context.Context, credentialID uuid.UUID, deltaMinor int64) error {
	if deltaMinor < 0 {
		return errors.New("deltaMinor must be positive")
	}
	s.mu.Lock()
	defer s.mu.Unlock()
	s.budgets[credentialID] += deltaMinor
	return nil
}

// SaveTransactionWithBudget atomically saves the transaction and increments the offline
// budget for credentialID under a single write lock.
// This eliminates the TOCTOU window that exists when SaveTransaction and
// UpdateOfflineBudget are called sequentially in the reconciliation service.
func (s *MemStore) SaveTransactionWithBudget(ctx context.Context, tx *domain.Transaction, credentialID uuid.UUID, deltaMinor int64) error {
	if deltaMinor < 0 {
		return errors.New("deltaMinor must be positive")
	}
	s.mu.Lock()
	defer s.mu.Unlock()
	cp := *tx
	s.transactions[tx.TxID] = &cp
	s.budgets[credentialID] += deltaMinor
	return nil
}
