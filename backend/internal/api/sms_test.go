package api

import (
	"bytes"
	"context"
	"encoding/base64"
	"encoding/hex"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/google/uuid"

	"github.com/Shikhyy/ResilientPay/backend/internal/domain"
)

// mockReconciler records calls and returns canned outcomes.
type mockSMSReconciler struct {
	outcome *domain.ReconciliationOutcome
	err     error
	called  bool
}

func (m *mockSMSReconciler) Reconcile(ctx context.Context, sub *domain.TransactionSubmission) (*domain.ReconciliationOutcome, error) {
	m.called = true
	return m.outcome, m.err
}

func TestSMSReassembler_RoundTrip(t *testing.T) {
	r := NewSMSReassembler(1 * time.Minute)

	const crossSDKCBORHex = "8d015000000000000000000000000000000001500000000000000000000000000000000250000000000000000000000000000000035000000000000000000000000000000004189663494e520150010101010101010101010101010101011a6553f1001a6553ff10f6f6"
	cborBytes, err := hex.DecodeString(crossSDKCBORHex)
	if err != nil {
		t.Fatalf("hex decode error: %v", err)
	}

	sigBytes := make([]byte, 64)
	for i := range sigBytes {
		sigBytes[i] = 0x5a
	}

	b64CBOR := base64.StdEncoding.EncodeToString(cborBytes)
	b64Sig := base64.StdEncoding.EncodeToString(sigBytes)

	// Ingest Part 1
	c1, s1, complete1, err1 := r.Ingest("RESPAY/1/2:test001:" + b64CBOR)
	if err1 != nil {
		t.Fatalf("part 1 ingest error: %v", err1)
	}
	if complete1 || c1 != nil || s1 != nil {
		t.Errorf("part 1 should not complete message")
	}

	// Ingest Part 2
	c2, s2, complete2, err2 := r.Ingest("RESPAY/2/2:test001:" + b64Sig)
	if err2 != nil {
		t.Fatalf("part 2 ingest error: %v", err2)
	}
	if !complete2 {
		t.Fatalf("part 2 should complete message")
	}

	if !bytes.Equal(c2, cborBytes) {
		t.Errorf("reassembled CBOR mismatch")
	}
	if !bytes.Equal(s2, sigBytes) {
		t.Errorf("reassembled signature mismatch")
	}
}

func TestSMSReassembler_OutOfOrder(t *testing.T) {
	r := NewSMSReassembler(1 * time.Minute)

	cborBytes := []byte("cbor-test-payload")
	sigBytes := []byte("sig-test-payload-64-bytes-00000000000000000000000000000000000000000000")

	b64CBOR := base64.StdEncoding.EncodeToString(cborBytes)
	b64Sig := base64.StdEncoding.EncodeToString(sigBytes)

	// Send Part 2 FIRST
	_, _, complete1, err1 := r.Ingest("RESPAY/2/2:test002:" + b64Sig)
	if err1 != nil || complete1 {
		t.Fatalf("part 2 first should succeed and not complete")
	}

	// Send Part 1 SECOND
	c2, s2, complete2, err2 := r.Ingest("RESPAY/1/2:test002:" + b64CBOR)
	if err2 != nil || !complete2 {
		t.Fatalf("part 1 second should complete")
	}

	if !bytes.Equal(c2, cborBytes) || !bytes.Equal(s2, sigBytes) {
		t.Errorf("out-of-order reassembly failed")
	}
}

func TestSMSReassembler_InvalidFormat(t *testing.T) {
	r := NewSMSReassembler(1 * time.Minute)

	// No RESPAY/ prefix
	_, _, _, err := r.Ingest("HELLO:123:abc")
	if err == nil {
		t.Errorf("expected error for missing RESPAY/ prefix")
	}

	// Bad part numbers
	_, _, _, err = r.Ingest("RESPAY/3/2:test:abc")
	if err == nil {
		t.Errorf("expected error for part > total")
	}
}

func TestHandleIngestSMS_HTTPIntegration(t *testing.T) {
	const crossSDKCBORHex = "8d015000000000000000000000000000000001500000000000000000000000000000000250000000000000000000000000000000035000000000000000000000000000000004189663494e520150010101010101010101010101010101011a6553f1001a6553ff10f6f6"
	cborBytes, _ := hex.DecodeString(crossSDKCBORHex)
	sigBytes := make([]byte, 64)

	b64CBOR := base64.StdEncoding.EncodeToString(cborBytes)
	b64Sig := base64.StdEncoding.EncodeToString(sigBytes)

	txID := uuid.MustParse("00000000-0000-0000-0000-000000000001")
	mock := &mockSMSReconciler{
		outcome: &domain.ReconciliationOutcome{
			TxID:   txID,
			Result: domain.ResultAccepted,
			Reason: "accepted",
		},
	}

	handler := NewHandler(mock, nil, "") // empty issuerSecret = dev mode
	mux := http.NewServeMux()
	handler.RegisterRoutes(mux)

	// 1. Post Part 1
	body1, _ := json.Marshal(SMSWebhookRequest{
		Sender:  "+919876543210",
		Message: "RESPAY/1/2:http001:" + b64CBOR,
	})
	req1 := httptest.NewRequest("POST", "/v1/telecom/sms", bytes.NewReader(body1))
	req1.Header.Set("Content-Type", "application/json")
	w1 := httptest.NewRecorder()
	mux.ServeHTTP(w1, req1)

	if w1.Code != http.StatusAccepted {
		t.Fatalf("expected 202 Accepted for part 1, got %d: %s", w1.Code, w1.Body.String())
	}

	// 2. Post Part 2
	body2, _ := json.Marshal(SMSWebhookRequest{
		Sender:  "+919876543210",
		Message: "RESPAY/2/2:http001:" + b64Sig,
	})
	req2 := httptest.NewRequest("POST", "/v1/telecom/sms", bytes.NewReader(body2))
	req2.Header.Set("Content-Type", "application/json")
	w2 := httptest.NewRecorder()
	mux.ServeHTTP(w2, req2)

	if w2.Code != http.StatusCreated {
		t.Fatalf("expected 201 Created for part 2, got %d: %s", w2.Code, w2.Body.String())
	}

	if !mock.called {
		t.Errorf("expected reconciler to be called upon reassembly")
	}
}
