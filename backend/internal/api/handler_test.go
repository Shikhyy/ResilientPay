// Package api — HTTP handler integration tests.
//
// Tests use net/http/httptest; no real network is needed.
package api

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/google/uuid"

	"github.com/Shikhyy/ResilientPay/backend/internal/domain"
	"github.com/Shikhyy/ResilientPay/backend/internal/store"
)

// ---------------------------------------------------------------------------
// Test double for the reconciler
// ---------------------------------------------------------------------------

type stubReconciler struct {
	outcome *domain.ReconciliationOutcome
	err     error
}

func (r *stubReconciler) Reconcile(_ context.Context, sub *domain.TransactionSubmission) (*domain.ReconciliationOutcome, error) {
	return r.outcome, r.err
}

func newTestHandler(outcome *domain.ReconciliationOutcome) (*Handler, *store.MemStore) {
	st := store.NewMemStore()
	h := NewHandler(&stubReconciler{outcome: outcome}, st)
	return h, st
}

func newMux(h *Handler) *http.ServeMux {
	mux := http.NewServeMux()
	h.RegisterRoutes(mux)
	return mux
}

// ---------------------------------------------------------------------------
// Health
// ---------------------------------------------------------------------------

func TestHealth_Returns200(t *testing.T) {
	h, _ := newTestHandler(nil)
	mux := newMux(h)

	req := httptest.NewRequest(http.MethodGet, "/v1/health", nil)
	rec := httptest.NewRecorder()
	mux.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Errorf("expected 200, got %d", rec.Code)
	}

	var body map[string]string
	_ = json.NewDecoder(rec.Body).Decode(&body)
	if body["status"] != "ok" {
		t.Errorf("expected status=ok, got %v", body)
	}
}

// ---------------------------------------------------------------------------
// Reconciliation endpoint
// ---------------------------------------------------------------------------

func TestReconcileEndpoint_AcceptedReturns201(t *testing.T) {
	txID := uuid.New()
	h, _ := newTestHandler(&domain.ReconciliationOutcome{
		TxID:   txID,
		Result: domain.ResultAccepted,
	})
	mux := newMux(h)

	sub := domain.TransactionSubmission{
		ProtocolVersion: domain.ProtocolVersion,
		TxID:            txID,
		CredentialID:    uuid.New(),
		PayerKeyID:      uuid.New(),
		MerchantID:      uuid.New(),
		AmountMinor:     100,
		Currency:        "INR",
		Counter:         1,
		Nonce:           make([]byte, 16),
		SignatureBytes:  make([]byte, 64),
	}
	body, _ := json.Marshal(sub)
	req := httptest.NewRequest(http.MethodPost, "/v1/reconciliation/transactions", bytes.NewReader(body))
	rec := httptest.NewRecorder()
	mux.ServeHTTP(rec, req)

	if rec.Code != http.StatusCreated {
		t.Errorf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestReconcileEndpoint_ConflictReturns409(t *testing.T) {
	txID := uuid.New()
	h, _ := newTestHandler(&domain.ReconciliationOutcome{
		TxID:   txID,
		Result: domain.ResultConflict,
		Reason: "conflicting evidence",
	})
	mux := newMux(h)

	body, _ := json.Marshal(map[string]any{"tx_id": txID})
	req := httptest.NewRequest(http.MethodPost, "/v1/reconciliation/transactions", bytes.NewReader(body))
	rec := httptest.NewRecorder()
	mux.ServeHTTP(rec, req)

	if rec.Code != http.StatusConflict {
		t.Errorf("expected 409, got %d", rec.Code)
	}
}

func TestReconcileEndpoint_InvalidJSONReturns400(t *testing.T) {
	h, _ := newTestHandler(nil)
	mux := newMux(h)

	req := httptest.NewRequest(http.MethodPost, "/v1/reconciliation/transactions", bytes.NewReader([]byte(`{broken json`)))
	rec := httptest.NewRecorder()
	mux.ServeHTTP(rec, req)

	if rec.Code != http.StatusBadRequest {
		t.Errorf("expected 400, got %d", rec.Code)
	}
}

// ---------------------------------------------------------------------------
// Transaction lookup
// ---------------------------------------------------------------------------

func TestGetTransaction_NotFoundReturns404(t *testing.T) {
	h, _ := newTestHandler(nil)
	mux := newMux(h)

	req := httptest.NewRequest(http.MethodGet, "/v1/transactions/"+uuid.New().String(), nil)
	rec := httptest.NewRecorder()
	mux.ServeHTTP(rec, req)

	if rec.Code != http.StatusNotFound {
		t.Errorf("expected 404, got %d", rec.Code)
	}
}

func TestGetTransaction_FoundReturns200(t *testing.T) {
	h, st := newTestHandler(nil)
	mux := newMux(h)
	ctx := context.Background()

	tx := &domain.Transaction{
		TxID:            uuid.New(),
		State:           domain.StateReconciled,
		Amount:          domain.Money{AmountMinor: 100, Currency: "INR"},
		Counter:         1,
		Nonce:           make([]byte, 16),
		ProtocolVersion: domain.ProtocolVersion,
	}
	_ = st.SaveTransaction(ctx, tx)

	req := httptest.NewRequest(http.MethodGet, "/v1/transactions/"+tx.TxID.String(), nil)
	rec := httptest.NewRecorder()
	mux.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Errorf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
}

// ---------------------------------------------------------------------------
// Credential lookup
// ---------------------------------------------------------------------------

func TestGetCredential_NotFoundReturns404(t *testing.T) {
	h, _ := newTestHandler(nil)
	mux := newMux(h)

	req := httptest.NewRequest(http.MethodGet, "/v1/credentials/"+uuid.New().String(), nil)
	rec := httptest.NewRecorder()
	mux.ServeHTTP(rec, req)

	if rec.Code != http.StatusNotFound {
		t.Errorf("expected 404, got %d", rec.Code)
	}
}
