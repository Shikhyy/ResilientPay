//go:build integration

package store

import (
	"context"
	"os"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/Shikhyy/ResilientPay/backend/internal/domain"
)

func TestPostgresStore_Integration(t *testing.T) {
	connString := os.Getenv("DATABASE_URL")
	if connString == "" {
		t.Skip("Skipping integration test: DATABASE_URL not set")
	}

	ctx := context.Background()
	s, err := NewPostgresStore(ctx, connString)
	if err != nil {
		t.Fatalf("Failed to connect to database: %v", err)
	}
	defer s.Close()

	// 1. Credential tests
	credID := uuid.New()
	cred := &domain.Credential{
		CredentialID:        credID,
		SubjectKeyID:        uuid.New(),
		PublicKeyBytes:      make([]byte, 32),
		IssuedAt:            time.Now().Truncate(time.Microsecond).UTC(), // pgx time precision
		ExpiresAt:           time.Now().Add(24 * time.Hour).Truncate(time.Microsecond).UTC(),
		MaxValuePerTxMinor:  5000,
		MaxValueOutstanding: 20000,
		MaxCounter:          100,
		State:               domain.CredentialActive,
		PolicyVersion:       1,
	}

	err = s.UpsertCredential(ctx, cred)
	if err != nil {
		t.Fatalf("Failed to upsert credential: %v", err)
	}

	retrievedCred, err := s.GetCredential(ctx, credID)
	if err != nil {
		t.Fatalf("Failed to get credential: %v", err)
	}
	if retrievedCred.CredentialID != credID {
		t.Errorf("Expected ID %s, got %s", credID, retrievedCred.CredentialID)
	}

	// 2. Budget tests
	budget, err := s.GetOfflineBudget(ctx, credID)
	if err != nil {
		t.Fatalf("Failed to get initial offline budget: %v", err)
	}
	if budget != 0 {
		t.Errorf("Expected initial budget 0, got %d", budget)
	}

	err = s.UpdateOfflineBudget(ctx, credID, 150)
	if err != nil {
		t.Fatalf("Failed to update offline budget: %v", err)
	}

	budget, err = s.GetOfflineBudget(ctx, credID)
	if err != nil {
		t.Fatalf("Failed to get updated offline budget: %v", err)
	}
	if budget != 150 {
		t.Errorf("Expected budget 150, got %d", budget)
	}

	// 3. Transaction tests
	txID := uuid.New()
	tx := &domain.Transaction{
		TxID:            txID,
		CredentialID:    credID,
		PayerKeyID:      uuid.New(),
		MerchantID:      uuid.New(),
		Amount:          domain.Money{AmountMinor: 150, Currency: "INR"},
		Counter:         1,
		Nonce:           make([]byte, 16),
		CreatedAtUnix:   time.Now().Unix(),
		ExpiresAtUnix:   time.Now().Add(1 * time.Hour).Unix(),
		SignatureBytes:  make([]byte, 64),
		ProtocolVersion: 1,
		State:           domain.StateReconciled,
	}

	err = s.SaveTransaction(ctx, tx)
	if err != nil {
		t.Fatalf("Failed to save transaction: %v", err)
	}

	retrievedTx, err := s.GetTransaction(ctx, txID)
	if err != nil {
		t.Fatalf("Failed to get transaction: %v", err)
	}
	if retrievedTx.TxID != txID {
		t.Errorf("Expected tx ID %s, got %s", txID, retrievedTx.TxID)
	}
}
