package settlement

import (
	"context"
	"log/slog"
	"time"

	"github.com/google/uuid"

	"github.com/Shikhyy/ResilientPay/backend/internal/domain"
	"github.com/Shikhyy/ResilientPay/backend/internal/store"
)

// Worker sweeps the database for RECONCILED transactions and transitions them to SETTLED.
// In a real system, this would interact with external banking APIs (e.g., UPI / IMPS rails)
// before marking them as settled. For the prototype, we simply transition the state.
type Worker struct {
	store    store.Store
	interval time.Duration
	batchSize int
	logger   *slog.Logger
}

// NewWorker creates a new settlement worker.
func NewWorker(s store.Store, interval time.Duration, batchSize int, logger *slog.Logger) *Worker {
	if logger == nil {
		logger = slog.Default()
	}
	return &Worker{
		store:     s,
		interval:  interval,
		batchSize: batchSize,
		logger:    logger.With("component", "SettlementWorker"),
	}
}

// Start runs the worker loop until the context is canceled.
func (w *Worker) Start(ctx context.Context) {
	w.logger.Info("settlement worker started", "interval", w.interval)
	ticker := time.NewTicker(w.interval)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			w.logger.Info("settlement worker stopping")
			return
		case <-ticker.C:
			w.processBatch(ctx)
		}
	}
}

func (w *Worker) processBatch(ctx context.Context) {
	txs, err := w.store.GetUnsettledTransactions(ctx, w.batchSize)
	if err != nil {
		w.logger.Error("failed to get unsettled transactions", "err", err)
		return
	}

	if len(txs) == 0 {
		return // Nothing to do
	}

	w.logger.Info("processing settlement batch", "count", len(txs))

	settledCount := 0
	for _, tx := range txs {
		// Mock API call to bank/clearing house would go here.
		// For the research prototype, we assume immediate settlement success.

		if err := w.store.MarkTransactionSettled(ctx, tx.TxID); err != nil {
			w.logger.Error("failed to settle transaction", "tx_id", tx.TxID, "err", err)
			continue
		}

		// Emit an audit event for the state change
		audit := &domain.AuditEvent{
			EventID:      uuid.New(), // Assuming we can use uuid.New() - wait we need to import uuid
			TxID:         &tx.TxID,
			CredentialID: &tx.CredentialID,
			Kind:       "SETTLEMENT_COMPLETED",
			Detail:     "transaction funds cleared",
			OccurredAt: time.Now().UTC(),
		}
		_ = w.store.RecordAuditEvent(ctx, audit)

		settledCount++
	}

	w.logger.Info("settlement batch complete", "settled", settledCount, "total", len(txs))
}
