package settlement

import (
	"context"
	"log/slog"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/Shikhyy/ResilientPay/backend/internal/domain"
	"github.com/Shikhyy/ResilientPay/backend/internal/store"
)

func TestWorker_ProcessBatch_SettlesReconciledTransactions(t *testing.T) {
	st := store.NewMemStore()
	
	// Create a dummy RECONCILED transaction
	txID := uuid.New()
	tx := &domain.Transaction{
		TxID:  txID,
		State: domain.StateReconciled,
	}
	_ = st.SaveTransaction(context.Background(), tx)

	worker := NewWorker(st, 1*time.Minute, 10, slog.Default())
	worker.processBatch(context.Background())

	// Verify the transaction is now SETTLED
	updatedTx, err := st.GetTransaction(context.Background(), txID)
	if err != nil {
		t.Fatalf("expected to find transaction, got err: %v", err)
	}

	if updatedTx.State != domain.StateSettled {
		t.Errorf("expected state to be SETTLED, got %s", updatedTx.State)
	}
}
