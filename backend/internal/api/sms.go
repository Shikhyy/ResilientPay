package api

import (
	"encoding/base64"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/Shikhyy/ResilientPay/backend/internal/crypto"
	"github.com/Shikhyy/ResilientPay/backend/internal/domain"
)

// SMSWebhookRequest represents an incoming SMS message forwarded by a telecom gateway.
type SMSWebhookRequest struct {
	Sender    string `json:"sender"`
	Recipient string `json:"recipient"`
	Message   string `json:"message"`
	Timestamp int64  `json:"timestamp,omitempty"`
}

// SMSBuffer stores partial fragments awaiting reassembly.
type SMSBuffer struct {
	TxPrefix  string
	Total     int
	Parts     map[int]string
	CreatedAt time.Time
}

// SMSReassembler handles thread-safe bounded in-memory reassembly of multipart SMS messages.
// Per ADR-006, stale fragments expire after a timeout to prevent memory exhaustion.
type SMSReassembler struct {
	mu      sync.Mutex
	buffers map[string]*SMSBuffer
	ttl     time.Duration
}

// NewSMSReassembler creates a new reassembler with specified TTL.
func NewSMSReassembler(ttl time.Duration) *SMSReassembler {
	r := &SMSReassembler{
		buffers: make(map[string]*SMSBuffer),
		ttl:     ttl,
	}
	return r
}

// Ingest processes an SMS text line: "RESPAY/<part>/<total>:<tx_prefix>:<payload>"
// Returns (cborBytes, sigBytes, complete, error).
func (r *SMSReassembler) Ingest(msg string) ([]byte, []byte, bool, error) {
	r.mu.Lock()
	defer r.mu.Unlock()

	// Clean up expired buffers
	now := time.Now()
	for k, buf := range r.buffers {
		if now.Sub(buf.CreatedAt) > r.ttl {
			delete(r.buffers, k)
		}
	}

	msg = strings.TrimSpace(msg)
	if !strings.HasPrefix(msg, "RESPAY/") {
		return nil, nil, false, errors.New("invalid SMS prefix; must start with RESPAY/")
	}

	// Format: RESPAY/<part>/<total>:<tx_prefix>:<payload>
	parts := strings.SplitN(msg, ":", 3)
	if len(parts) != 3 {
		return nil, nil, false, errors.New("invalid SMS format; expected RESPAY/<part>/<total>:<prefix>:<payload>")
	}

	header := strings.TrimPrefix(parts[0], "RESPAY/")
	headerParts := strings.Split(header, "/")
	if len(headerParts) != 2 {
		return nil, nil, false, errors.New("invalid header; expected <part>/<total>")
	}

	partNum, err := strconv.Atoi(headerParts[0])
	if err != nil || partNum < 1 {
		return nil, nil, false, errors.New("invalid part number")
	}
	totalNum, err := strconv.Atoi(headerParts[1])
	if err != nil || totalNum < 1 || totalNum > 5 {
		return nil, nil, false, errors.New("invalid total parts; bounded 1..5")
	}
	if partNum > totalNum {
		return nil, nil, false, errors.New("part number exceeds total parts")
	}

	txPrefix := parts[1]
	payload := parts[2]

	buf, exists := r.buffers[txPrefix]
	if !exists {
		buf = &SMSBuffer{
			TxPrefix:  txPrefix,
			Total:     totalNum,
			Parts:     make(map[int]string),
			CreatedAt: now,
		}
		r.buffers[txPrefix] = buf
	}

	if buf.Total != totalNum {
		return nil, nil, false, errors.New("inconsistent total parts for prefix")
	}

	buf.Parts[partNum] = payload

	if len(buf.Parts) == buf.Total {
		// Complete! Part 1 is CBOR, Part 2 is Signature
		cborStr := buf.Parts[1]
		sigStr := buf.Parts[2]

		delete(r.buffers, txPrefix)

		cborBytes, err := base64.StdEncoding.DecodeString(cborStr)
		if err != nil {
			return nil, nil, false, fmt.Errorf("failed to decode base64 CBOR: %w", err)
		}

		sigBytes, err := base64.StdEncoding.DecodeString(sigStr)
		if err != nil {
			return nil, nil, false, fmt.Errorf("failed to decode base64 signature: %w", err)
		}

		return cborBytes, sigBytes, true, nil
	}

	return nil, nil, false, nil
}

// Global default reassembler for the API handler
var defaultSMSReassembler = NewSMSReassembler(10 * time.Minute)

// handleIngestSMS receives an SMS webhook payload, reassembles multipart segments,
// and reconciles completed transaction envelopes.
func (h *Handler) handleIngestSMS(w http.ResponseWriter, r *http.Request) {
	var req SMSWebhookRequest

	contentType := r.Header.Get("Content-Type")
	if strings.Contains(contentType, "application/json") {
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			writeError(w, http.StatusBadRequest, "invalid JSON body", "INVALID_REQUEST")
			return
		}
	} else {
		// Assume text/plain
		var bodyBuf strings.Builder
		buf := make([]byte, 1024)
		for {
			n, err := r.Body.Read(buf)
			if n > 0 {
				bodyBuf.Write(buf[:n])
			}
			if err != nil {
				break
			}
		}
		req.Message = bodyBuf.String()
	}

	if req.Message == "" {
		writeError(w, http.StatusBadRequest, "empty SMS message body", "INVALID_MESSAGE")
		return
	}

	cborBytes, sigBytes, complete, err := defaultSMSReassembler.Ingest(req.Message)
	if err != nil {
		writeError(w, http.StatusBadRequest, err.Error(), "SMS_PARSE_ERROR")
		return
	}

	if !complete {
		writeJSON(w, http.StatusAccepted, map[string]interface{}{
			"status":  "FRAGMENT_ACCEPTED",
			"message": "Partial fragment stored, awaiting remaining SMS parts",
		})
		return
	}

	// Reconstruct TransactionSubmission from canonical CBOR
	sub, err := crypto.DecodeFromCBOR(cborBytes)
	if err != nil {
		writeError(w, http.StatusBadRequest, fmt.Sprintf("invalid CBOR envelope: %v", err), "CBOR_DECODE_FAILED")
		return
	}
	sub.SignatureBytes = sigBytes

	// Reconcile
	ctx := r.Context()
	outcome, err := h.reconciler.Reconcile(ctx, sub)
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

	writeJSON(w, status, map[string]interface{}{
		"status":  "COMPLETED",
		"outcome": outcome,
	})
}
