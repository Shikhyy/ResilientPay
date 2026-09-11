package store

import (
	"context"
	"errors"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"

	"github.com/Shikhyy/ResilientPay/backend/internal/domain"
)

// PostgresStore implements the Store interface using PostgreSQL.
type PostgresStore struct {
	pool *pgxpool.Pool
}

// Compile-time check to ensure PostgresStore implements Store.
var _ Store = (*PostgresStore)(nil)

// NewPostgresStore creates a new PostgreSQL-backed store.
func NewPostgresStore(ctx context.Context, connString string) (*PostgresStore, error) {
	pool, err := pgxpool.New(ctx, connString)
	if err != nil {
		return nil, err
	}
	// Verify connection
	if err := pool.Ping(ctx); err != nil {
		return nil, err
	}
	return &PostgresStore{pool: pool}, nil
}

// Close closes the connection pool.
func (s *PostgresStore) Close() {
	s.pool.Close()
}

// GetCredential retrieves a credential by ID.
func (s *PostgresStore) GetCredential(ctx context.Context, id uuid.UUID) (*domain.Credential, error) {
	var c domain.Credential
	err := s.pool.QueryRow(ctx, `
		SELECT credential_id, subject_key_id, public_key_bytes, issued_at, expires_at,
		       max_value_per_tx_minor, max_value_outstanding_minor, max_counter, state, policy_version
		FROM credentials
		WHERE credential_id = $1
	`, id).Scan(
		&c.CredentialID, &c.SubjectKeyID, &c.PublicKeyBytes, &c.IssuedAt, &c.ExpiresAt,
		&c.MaxValuePerTxMinor, &c.MaxValueOutstanding, &c.MaxCounter, &c.State, &c.PolicyVersion,
	)
	if errors.Is(err, pgx.ErrNoRows) {
		return nil, ErrNotFound
	}
	if err != nil {
		return nil, err
	}
	return &c, nil
}

// UpsertCredential inserts or updates a credential.
func (s *PostgresStore) UpsertCredential(ctx context.Context, c *domain.Credential) error {
	_, err := s.pool.Exec(ctx, `
		INSERT INTO credentials (
			credential_id, subject_key_id, public_key_bytes, issued_at, expires_at,
			max_value_per_tx_minor, max_value_outstanding_minor, max_counter, state, policy_version
		) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
		ON CONFLICT (credential_id) DO UPDATE SET
			state = EXCLUDED.state,
			policy_version = EXCLUDED.policy_version,
			updated_at = NOW()
	`, c.CredentialID, c.SubjectKeyID, c.PublicKeyBytes, c.IssuedAt, c.ExpiresAt,
		c.MaxValuePerTxMinor, c.MaxValueOutstanding, c.MaxCounter, c.State, c.PolicyVersion)
	return err
}

// GetTransaction retrieves a transaction by ID.
func (s *PostgresStore) GetTransaction(ctx context.Context, txID uuid.UUID) (*domain.Transaction, error) {
	var tx domain.Transaction
	err := s.pool.QueryRow(ctx, `
		SELECT tx_id, credential_id, payer_key_id, merchant_id,
		       amount_minor, currency, counter, nonce,
		       created_at_unix, expires_at_unix, previous_event_hash, risk_class,
		       signature_bytes, protocol_version, state
		FROM transactions
		WHERE tx_id = $1
	`, txID).Scan(
		&tx.TxID, &tx.CredentialID, &tx.PayerKeyID, &tx.MerchantID,
		&tx.Amount.AmountMinor, &tx.Amount.Currency, &tx.Counter, &tx.Nonce,
		&tx.CreatedAtUnix, &tx.ExpiresAtUnix, &tx.PreviousEventHash, &tx.RiskClass,
		&tx.SignatureBytes, &tx.ProtocolVersion, &tx.State,
	)
	if errors.Is(err, pgx.ErrNoRows) {
		return nil, ErrNotFound
	}
	if err != nil {
		return nil, err
	}
	return &tx, nil
}

// SaveTransaction inserts or updates a transaction idempotently.
func (s *PostgresStore) SaveTransaction(ctx context.Context, tx *domain.Transaction) error {
	_, err := s.pool.Exec(ctx, `
		INSERT INTO transactions (
			tx_id, credential_id, payer_key_id, merchant_id,
			amount_minor, currency, counter, nonce,
			created_at_unix, expires_at_unix, previous_event_hash, risk_class,
			signature_bytes, protocol_version, state
		) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
		ON CONFLICT (tx_id) DO UPDATE SET
			state = EXCLUDED.state,
			updated_at = NOW()
	`, tx.TxID, tx.CredentialID, tx.PayerKeyID, tx.MerchantID,
		tx.Amount.AmountMinor, tx.Amount.Currency, tx.Counter, tx.Nonce,
		tx.CreatedAtUnix, tx.ExpiresAtUnix, tx.PreviousEventHash, tx.RiskClass,
		tx.SignatureBytes, tx.ProtocolVersion, tx.State)
	return err
}

// RecordAuditEvent logs a security or operational event.
func (s *PostgresStore) RecordAuditEvent(ctx context.Context, ev *domain.AuditEvent) error {
	_, err := s.pool.Exec(ctx, `
		INSERT INTO audit_events (tx_id, credential_id, kind, detail, occurred_at)
		VALUES ($1, $2, $3, $4, $5)
	`, ev.TxID, ev.CredentialID, ev.Kind, ev.Detail, ev.OccurredAt)
	return err
}

// ListAuditEvents lists audit events for a transaction.
func (s *PostgresStore) ListAuditEvents(ctx context.Context, txID *uuid.UUID) ([]*domain.AuditEvent, error) {
	rows, err := s.pool.Query(ctx, `
		SELECT event_id, tx_id, credential_id, kind, detail, occurred_at
		FROM audit_events
		WHERE ($1::uuid IS NULL OR tx_id = $1)
		ORDER BY occurred_at ASC
	`, txID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var events []*domain.AuditEvent
	for rows.Next() {
		var ev domain.AuditEvent
		if err := rows.Scan(&ev.EventID, &ev.TxID, &ev.CredentialID, &ev.Kind, &ev.Detail, &ev.OccurredAt); err != nil {
			return nil, err
		}
		events = append(events, &ev)
	}
	if err := rows.Err(); err != nil {
		return nil, err
	}
	return events, nil
}

// GetOfflineBudget retrieves the accumulated offline budget for a credential.
func (s *PostgresStore) GetOfflineBudget(ctx context.Context, credentialID uuid.UUID) (int64, error) {
	var outstanding int64
	err := s.pool.QueryRow(ctx, `
		SELECT outstanding_minor
		FROM offline_budgets
		WHERE credential_id = $1
	`, credentialID).Scan(&outstanding)
	if errors.Is(err, pgx.ErrNoRows) {
		return 0, nil // No budget recorded yet is valid (0 outstanding)
	}
	return outstanding, err
}

// UpdateOfflineBudget increases the offline budget for a credential by deltaMinor.
func (s *PostgresStore) UpdateOfflineBudget(ctx context.Context, credentialID uuid.UUID, deltaMinor int64) error {
	if deltaMinor < 0 {
		return errors.New("deltaMinor must be positive")
	}
	_, err := s.pool.Exec(ctx, `
		INSERT INTO offline_budgets (credential_id, outstanding_minor)
		VALUES ($1, $2)
		ON CONFLICT (credential_id) DO UPDATE SET
			outstanding_minor = offline_budgets.outstanding_minor + EXCLUDED.outstanding_minor,
			last_updated_at = NOW()
	`, credentialID, deltaMinor)
	return err
}

func (s *PostgresStore) GetUnsettledTransactions(ctx context.Context, limit int) ([]*domain.Transaction, error) {
	rows, err := s.pool.Query(ctx, `
		SELECT tx_id, credential_id, payer_key_id, merchant_id,
		       amount_minor, currency, counter, nonce,
		       created_at_unix, expires_at_unix, signature_bytes, state
		FROM transactions
		WHERE state = 'RECONCILED'
		ORDER BY created_at_unix ASC
		LIMIT $1
	`, limit)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var txs []*domain.Transaction
	for rows.Next() {
		var tx domain.Transaction
		if err := rows.Scan(
			&tx.TxID, &tx.CredentialID, &tx.PayerKeyID, &tx.MerchantID,
			&tx.Amount.AmountMinor, &tx.Amount.Currency, &tx.Counter, &tx.Nonce,
			&tx.CreatedAtUnix, &tx.ExpiresAtUnix, &tx.SignatureBytes, &tx.State,
		); err != nil {
			return nil, err
		}
		txs = append(txs, &tx)
	}
	return txs, rows.Err()
}

func (s *PostgresStore) MarkTransactionSettled(ctx context.Context, txID uuid.UUID) error {
	res, err := s.pool.Exec(ctx, `
		UPDATE transactions
		SET state = 'SETTLED'
		WHERE tx_id = $1 AND state = 'RECONCILED'
	`, txID)
	if err != nil {
		return err
	}
	if res.RowsAffected() == 0 {
		return ErrNotFound
	}
	return nil
}
