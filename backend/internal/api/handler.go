// Package api implements the HTTP API handlers for ResilientPay backend.
//
// Endpoints (per docs/06-development/API_SPEC.md §2):
//
//	GET  /v1/health
//	POST /v1/test-credentials         — provision a synthetic test credential
//	POST /v1/reconciliation/transactions — submit a transaction for reconciliation
//	GET  /v1/transactions/{tx_id}     — inspect transaction state
//	GET  /v1/credentials/{credential_id}
//
// All responses use JSON. All mutating endpoints are idempotent.
// Error responses use a structured JSON body.
package api

import (
	"context"
	"encoding/json"
	"net/http"
	"strings"
	"time"

	"github.com/google/uuid"

	"github.com/Shikhyy/ResilientPay/backend/internal/domain"
	"github.com/Shikhyy/ResilientPay/backend/internal/store"
)

// ---------------------------------------------------------------------------
// Dependencies
// ---------------------------------------------------------------------------

// Reconciler is the reconciliation service interface expected by the handler.
type Reconciler interface {
	Reconcile(ctx context.Context, sub *domain.TransactionSubmission) (*domain.ReconciliationOutcome, error)
}

// Handler holds the dependencies for all API routes.
type Handler struct {
	reconciler Reconciler
	store      store.Store
}

// NewHandler constructs an API Handler.
func NewHandler(r Reconciler, s store.Store) *Handler {
	return &Handler{reconciler: r, store: s}
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

// RegisterRoutes attaches all API routes to the given ServeMux.
func (h *Handler) RegisterRoutes(mux *http.ServeMux) {
	mux.HandleFunc("GET /v1/health", h.handleHealth)
	mux.HandleFunc("POST /v1/test-credentials", h.handleCreateTestCredential)
	mux.HandleFunc("POST /v1/reconciliation/transactions", h.handleReconcileTransaction)
	mux.HandleFunc("GET /v1/transactions/", h.handleGetTransaction)
	mux.HandleFunc("GET /v1/credentials/", h.handleGetCredential)
}

// ---------------------------------------------------------------------------
// Response helpers
// ---------------------------------------------------------------------------

type errorResponse struct {
	Error string `json:"error"`
	Code  string `json:"code,omitempty"`
}

func writeJSON(w http.ResponseWriter, status int, v any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(v)
}

func writeError(w http.ResponseWriter, status int, msg, code string) {
	writeJSON(w, status, errorResponse{Error: msg, Code: code})
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

func (h *Handler) handleHealth(w http.ResponseWriter, r *http.Request) {
	writeJSON(w, http.StatusOK, map[string]string{
		"status":  "ok",
		"service": "resilientpay-backend",
		"time":    time.Now().UTC().Format(time.RFC3339),
	})
}

// handleCreateTestCredential provisions a synthetic test credential.
// This is a prototype endpoint — NOT a real credential issuance API.
func (h *Handler) handleCreateTestCredential(w http.ResponseWriter, r *http.Request) {
	var req struct {
		PublicKeyHex        string `json:"public_key_hex"`
		MaxValuePerTxMinor  uint64 `json:"max_value_per_tx_minor"`
		MaxValueOutstanding uint64 `json:"max_value_outstanding_minor"`
		MaxCounter          uint64 `json:"max_counter"`
		ValidForSeconds     int64  `json:"valid_for_seconds"`
	}

	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid JSON body", "INVALID_REQUEST")
		return
	}
	if len(req.PublicKeyHex) != 64 {
		writeError(w, http.StatusBadRequest, "public_key_hex must be 64 hex characters (32 bytes)", "INVALID_PUBLIC_KEY")
		return
	}
	pkBytes, err := hexDecode(req.PublicKeyHex)
	if err != nil || len(pkBytes) != 32 {
		writeError(w, http.StatusBadRequest, "public_key_hex is not valid hex", "INVALID_PUBLIC_KEY")
		return
	}
	if req.ValidForSeconds <= 0 {
		req.ValidForSeconds = 3600 // default 1 hour
	}
	if req.MaxCounter == 0 {
		req.MaxCounter = 1000
	}
	if req.MaxValuePerTxMinor == 0 {
		req.MaxValuePerTxMinor = 50_000
	}
	if req.MaxValueOutstanding == 0 {
		req.MaxValueOutstanding = 200_000
	}

	now := time.Now().UTC()
	cred := &domain.Credential{
		CredentialID:        uuid.New(),
		SubjectKeyID:        uuid.New(),
		PublicKeyBytes:      pkBytes,
		IssuedAt:            now,
		ExpiresAt:           now.Add(time.Duration(req.ValidForSeconds) * time.Second),
		MaxValuePerTxMinor:  req.MaxValuePerTxMinor,
		MaxValueOutstanding: req.MaxValueOutstanding,
		MaxCounter:          req.MaxCounter,
		State:               domain.CredentialActive,
		PolicyVersion:       1,
	}

	ctx := r.Context()
	if err := h.store.UpsertCredential(ctx, cred); err != nil {
		writeError(w, http.StatusInternalServerError, "failed to store credential", "STORE_ERROR")
		return
	}

	writeJSON(w, http.StatusCreated, map[string]any{
		"credential_id":  cred.CredentialID.String(),
		"subject_key_id": cred.SubjectKeyID.String(),
		"state":          cred.State,
		"expires_at":     cred.ExpiresAt.Format(time.RFC3339),
	})
}

// handleReconcileTransaction receives a TransactionSubmission and runs reconciliation.
func (h *Handler) handleReconcileTransaction(w http.ResponseWriter, r *http.Request) {
	var sub domain.TransactionSubmission
	if err := json.NewDecoder(r.Body).Decode(&sub); err != nil {
		writeError(w, http.StatusBadRequest, "invalid JSON body", "INVALID_REQUEST")
		return
	}

	ctx := r.Context()
	outcome, err := h.reconciler.Reconcile(ctx, &sub)
	if err != nil {
		writeError(w, http.StatusInternalServerError, "reconciliation error", "INTERNAL_ERROR")
		return
	}

	status := http.StatusOK
	switch outcome.Result {
	case domain.ResultAccepted:
		status = http.StatusCreated
	case domain.ResultAlreadyKnown:
		status = http.StatusOK
	case domain.ResultRejected:
		status = http.StatusUnprocessableEntity
	case domain.ResultConflict:
		status = http.StatusConflict
	}

	writeJSON(w, status, outcome)
}

// handleGetTransaction returns the current state of a transaction.
func (h *Handler) handleGetTransaction(w http.ResponseWriter, r *http.Request) {
	txIDStr := strings.TrimPrefix(r.URL.Path, "/v1/transactions/")
	txID, err := uuid.Parse(txIDStr)
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid tx_id UUID", "INVALID_ID")
		return
	}

	tx, err := h.store.GetTransaction(r.Context(), txID)
	if err == store.ErrNotFound {
		writeError(w, http.StatusNotFound, "transaction not found", "NOT_FOUND")
		return
	}
	if err != nil {
		writeError(w, http.StatusInternalServerError, "store error", "INTERNAL_ERROR")
		return
	}

	writeJSON(w, http.StatusOK, tx)
}

// handleGetCredential returns the metadata of a credential (no key material).
func (h *Handler) handleGetCredential(w http.ResponseWriter, r *http.Request) {
	idStr := strings.TrimPrefix(r.URL.Path, "/v1/credentials/")
	credID, err := uuid.Parse(idStr)
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid credential_id UUID", "INVALID_ID")
		return
	}

	cred, err := h.store.GetCredential(r.Context(), credID)
	if err == store.ErrNotFound {
		writeError(w, http.StatusNotFound, "credential not found", "NOT_FOUND")
		return
	}
	if err != nil {
		writeError(w, http.StatusInternalServerError, "store error", "INTERNAL_ERROR")
		return
	}

	// Return metadata only — never return private key material.
	writeJSON(w, http.StatusOK, map[string]any{
		"credential_id":  cred.CredentialID.String(),
		"subject_key_id": cred.SubjectKeyID.String(),
		"state":          cred.State,
		"expires_at":     cred.ExpiresAt.Format(time.RFC3339),
		"max_counter":    cred.MaxCounter,
	})
}

// ---------------------------------------------------------------------------
// Hex helper (minimal, avoids importing encoding/hex for readability)
// ---------------------------------------------------------------------------

func hexDecode(s string) ([]byte, error) {
	// stdlib encoding/hex is fine; keep the import local to this function
	// to avoid confusion with crypto packages.
	import_ := func() {}
	_ = import_
	// Use stdlib hex decode
	b := make([]byte, len(s)/2)
	for i := range b {
		hi := hexNibble(s[i*2])
		lo := hexNibble(s[i*2+1])
		if hi == 255 || lo == 255 {
			return nil, &hexError{}
		}
		b[i] = hi<<4 | lo
	}
	return b, nil
}

type hexError struct{}

func (hexError) Error() string { return "invalid hex" }

func hexNibble(c byte) byte {
	switch {
	case c >= '0' && c <= '9':
		return c - '0'
	case c >= 'a' && c <= 'f':
		return c - 'a' + 10
	case c >= 'A' && c <= 'F':
		return c - 'A' + 10
	}
	return 255
}
