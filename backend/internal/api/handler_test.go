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
	h := NewHandler(&stubReconciler{outcome: outcome}, st, "") // empty secret = dev mode
	return h, st
}

// newTestHandlerWithSecret creates a handler with an issuer secret configured.
func newTestHandlerWithSecret(outcome *domain.ReconciliationOutcome, secret string) (*Handler, *store.MemStore) {
	st := store.NewMemStore()
	h := NewHandler(&stubReconciler{outcome: outcome}, st, secret)
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

		body, _ := json.Marshal(map[string]any{
		"protocol_version": 1,
		"tx_id":            txID,
		"credential_id":    uuid.New(),
		"payer_key_id":     uuid.New(),
		"merchant_id":      uuid.New(),
		"amount_minor":     100,
		"currency":         "INR",
		"counter":          1,
		"nonce":            "MDAwMDAwMDAwMDAwMDAwMA==",
		"signature_bytes":  "MDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMA==",
	})
	req := httptest.NewRequest(http.MethodPost, "/v1/reconciliation/transactions", bytes.NewReader(body))
	rec := httptest.NewRecorder()
	mux.ServeHTTP(rec, req)

	if rec.Code != http.StatusConflict {
		t.Errorf("expected 409, got %d, body: %s", rec.Code, rec.Body.String())
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

func TestIssueCredential_InvalidSchemaReturns400(t *testing.T) {
	h, _ := newTestHandler(nil)
	mux := newMux(h)

	body := `{"public_key_hex": "123"}` // Invalid, needs to be 64 chars
	req := httptest.NewRequest(http.MethodPost, "/v1/credentials", bytes.NewReader([]byte(body)))
	rec := httptest.NewRecorder()
	mux.ServeHTTP(rec, req)

	if rec.Code != http.StatusBadRequest {
		t.Errorf("expected 400, got %d", rec.Code)
	}
}

func TestIssueCredential_ValidReturns201(t *testing.T) {
	h, _ := newTestHandler(nil)
	mux := newMux(h)

	body := `{
		"subject_key_id_hex": "00000000000000000000000000000000",
		"public_key_hex": "0000000000000000000000000000000000000000000000000000000000000000",
		"max_value_per_tx_minor": 50000,
		"max_value_outstanding_minor": 200000,
		"max_counter": 1000,
		"valid_for_seconds": 3600
	}`
	req := httptest.NewRequest(http.MethodPost, "/v1/credentials", bytes.NewReader([]byte(body)))
	rec := httptest.NewRecorder()
	mux.ServeHTTP(rec, req)

	if rec.Code != http.StatusCreated {
		t.Errorf("expected 201, got %d. body: %s", rec.Code, rec.Body.String())
	}
}

// ---------------------------------------------------------------------------
// Credential revocation
// ---------------------------------------------------------------------------

const validIssueBody = `{
	"subject_key_id_hex": "00000000000000000000000000000000",
	"public_key_hex": "0000000000000000000000000000000000000000000000000000000000000000",
	"max_value_per_tx_minor": 50000,
	"max_value_outstanding_minor": 200000,
	"max_counter": 1000,
	"valid_for_seconds": 3600
}`

// issueCredential is a test helper that issues a credential and returns its ID string.
func issueCredential(t *testing.T, mux *http.ServeMux) string {
	t.Helper()
	req := httptest.NewRequest(http.MethodPost, "/v1/credentials", bytes.NewReader([]byte(validIssueBody)))
	rec := httptest.NewRecorder()
	mux.ServeHTTP(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("issueCredential: expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
	var body map[string]any
	if err := json.NewDecoder(rec.Body).Decode(&body); err != nil {
		t.Fatalf("issueCredential: decode body: %v", err)
	}
	id, ok := body["credential_id"].(string)
	if !ok || id == "" {
		t.Fatalf("issueCredential: missing credential_id in response: %v", body)
	}
	return id
}

func TestRevokeCredential_ActiveReturns200(t *testing.T) {
	h, _ := newTestHandler(nil)
	mux := newMux(h)

	credID := issueCredential(t, mux)

	req := httptest.NewRequest(http.MethodPost, "/v1/credentials/"+credID+"/revoke", nil)
	rec := httptest.NewRecorder()
	mux.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}

	var body map[string]string
	if err := json.NewDecoder(rec.Body).Decode(&body); err != nil {
		t.Fatalf("decode response: %v", err)
	}
	if body["state"] != "REVOKED" {
		t.Errorf("expected state=REVOKED, got %q", body["state"])
	}
	if body["credential_id"] != credID {
		t.Errorf("expected credential_id=%s, got %q", credID, body["credential_id"])
	}
}

func TestRevokeCredential_AlreadyRevokedReturns409(t *testing.T) {
	h, _ := newTestHandler(nil)
	mux := newMux(h)

	credID := issueCredential(t, mux)

	// First revoke — must succeed.
	req1 := httptest.NewRequest(http.MethodPost, "/v1/credentials/"+credID+"/revoke", nil)
	rec1 := httptest.NewRecorder()
	mux.ServeHTTP(rec1, req1)
	if rec1.Code != http.StatusOK {
		t.Fatalf("first revoke: expected 200, got %d: %s", rec1.Code, rec1.Body.String())
	}

	// Second revoke — must return 409.
	req2 := httptest.NewRequest(http.MethodPost, "/v1/credentials/"+credID+"/revoke", nil)
	rec2 := httptest.NewRecorder()
	mux.ServeHTTP(rec2, req2)
	if rec2.Code != http.StatusConflict {
		t.Errorf("second revoke: expected 409, got %d: %s", rec2.Code, rec2.Body.String())
	}

	var body map[string]string
	if err := json.NewDecoder(rec2.Body).Decode(&body); err != nil {
		t.Fatalf("decode 409 response: %v", err)
	}
	if body["code"] != "ALREADY_REVOKED" {
		t.Errorf("expected code=ALREADY_REVOKED, got %q", body["code"])
	}
}

func TestRevokeCredential_NotFoundReturns404(t *testing.T) {
	h, _ := newTestHandler(nil)
	mux := newMux(h)

	fakeID := "00000000-0000-0000-0000-000000000099"
	req := httptest.NewRequest(http.MethodPost, "/v1/credentials/"+fakeID+"/revoke", nil)
	rec := httptest.NewRecorder()
	mux.ServeHTTP(rec, req)

	if rec.Code != http.StatusNotFound {
		t.Errorf("expected 404, got %d: %s", rec.Code, rec.Body.String())
	}
}

// ---------------------------------------------------------------------------
// Issuer bearer token authentication
// ---------------------------------------------------------------------------

func TestIssuerAuth_MissingTokenReturns401(t *testing.T) {
	h, _ := newTestHandlerWithSecret(nil, "supersecret")
	mux := newMux(h)

	req := httptest.NewRequest(http.MethodPost, "/v1/credentials", bytes.NewReader([]byte(validIssueBody)))
	// No Authorization header.
	rec := httptest.NewRecorder()
	mux.ServeHTTP(rec, req)

	if rec.Code != http.StatusUnauthorized {
		t.Errorf("expected 401, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestIssuerAuth_WrongTokenReturns401(t *testing.T) {
	h, _ := newTestHandlerWithSecret(nil, "supersecret")
	mux := newMux(h)

	req := httptest.NewRequest(http.MethodPost, "/v1/credentials", bytes.NewReader([]byte(validIssueBody)))
	req.Header.Set("Authorization", "Bearer wrongtoken")
	rec := httptest.NewRecorder()
	mux.ServeHTTP(rec, req)

	if rec.Code != http.StatusUnauthorized {
		t.Errorf("expected 401, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestIssuerAuth_CorrectTokenReturns201(t *testing.T) {
	h, _ := newTestHandlerWithSecret(nil, "supersecret")
	mux := newMux(h)

	req := httptest.NewRequest(http.MethodPost, "/v1/credentials", bytes.NewReader([]byte(validIssueBody)))
	req.Header.Set("Authorization", "Bearer supersecret")
	rec := httptest.NewRecorder()
	mux.ServeHTTP(rec, req)

	if rec.Code != http.StatusCreated {
		t.Errorf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestIssuerAuth_NoEnvVarAllowsRequest(t *testing.T) {
	// Empty issuerSecret = dev mode; all requests pass through without auth.
	h, _ := newTestHandler(nil)
	mux := newMux(h)

	req := httptest.NewRequest(http.MethodPost, "/v1/credentials", bytes.NewReader([]byte(validIssueBody)))
	// No Authorization header — must still succeed because the secret is not configured.
	rec := httptest.NewRecorder()
	mux.ServeHTTP(rec, req)

	if rec.Code != http.StatusCreated {
		t.Errorf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
}
