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
	// Task 3: Formal issuance API (replaces test-credentials)
	mux.HandleFunc("POST /v1/credentials", h.handleIssueCredential)
	// Task 3: Reconcile handler
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

// handleIssueCredential provisions a new OfflineCredential for a device securely.
func (h *Handler) handleIssueCredential(w http.ResponseWriter, r *http.Request) {
	var req IssueCredentialRequest

	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid JSON body", "INVALID_REQUEST")
		return
	}

	if err := req.Validate(); err != nil {
		writeError(w, http.StatusBadRequest, err.Error(), "VALIDATION_FAILED")
		return
	}

	pkBytes, err := hexDecode(req.PublicKeyHex)
	if err != nil {
		writeError(w, http.StatusBadRequest, "public_key_hex is not valid hex", "INVALID_PUBLIC_KEY")
		return
	}

	subKeyBytes, err := hexDecode(req.SubjectKeyIDHex)
	if err != nil {
		writeError(w, http.StatusBadRequest, "subject_key_id_hex is not valid hex", "INVALID_SUBJECT_KEY")
		return
	}
	subjectKeyID, err := uuid.FromBytes(subKeyBytes)
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid subject key UUID bytes", "INVALID_SUBJECT_KEY")
		return
	}

	now := time.Now().UTC()
	cred := &domain.Credential{
		CredentialID:        uuid.New(),
		SubjectKeyID:        subjectKeyID,
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
		// We explicitly return the parameters so the device can construct its local
		// OfflineCredential structurally.
		"issued_at":                   cred.IssuedAt.Format(time.RFC3339),
		"max_value_per_tx_minor":      cred.MaxValuePerTxMinor,
		"max_value_outstanding_minor": cred.MaxValueOutstanding,
		"max_counter":                 cred.MaxCounter,
		"policy_version":              cred.PolicyVersion,
	})
}

// handleReconcileTransaction receives a TransactionSubmission and runs reconciliation.
func (h *Handler) handleReconcileTransaction(w http.ResponseWriter, r *http.Request) {
	var sub domain.TransactionSubmission
	if err := json.NewDecoder(r.Body).Decode(&sub); err != nil {
		writeError(w, http.StatusBadRequest, "invalid JSON body", "INVALID_REQUEST")
		return
	}

	if err := ValidateSubmission(&sub); err != nil {
		writeError(w, http.StatusBadRequest, err.Error(), "VALIDATION_FAILED")
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
