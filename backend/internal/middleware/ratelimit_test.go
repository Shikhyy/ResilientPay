package middleware

import (
	"bytes"
	"fmt"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/google/uuid"
)

func TestRateLimiter_AllowsUnderLimit(t *testing.T) {
	rl := NewRateLimiter(10, 2) // 10 req/s, burst 2
	credID := uuid.New()

	handler := rl.Handler(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))

	body := []byte(fmt.Sprintf(`{"credential_id":"%s"}`, credID.String()))

	// First request should pass
	req1 := httptest.NewRequest(http.MethodPost, "/reconcile", bytes.NewReader(body))
	rec1 := httptest.NewRecorder()
	handler.ServeHTTP(rec1, req1)
	if rec1.Code != http.StatusOK {
		t.Errorf("Expected 200 OK, got %d", rec1.Code)
	}

	// Second request should pass (burst=2)
	req2 := httptest.NewRequest(http.MethodPost, "/reconcile", bytes.NewReader(body))
	rec2 := httptest.NewRecorder()
	handler.ServeHTTP(rec2, req2)
	if rec2.Code != http.StatusOK {
		t.Errorf("Expected 200 OK, got %d", rec2.Code)
	}
}

func TestRateLimiter_BlocksOverLimit(t *testing.T) {
	rl := NewRateLimiter(10, 1) // 10 req/s, burst 1
	credID := uuid.New()

	handler := rl.Handler(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))

	body := []byte(fmt.Sprintf(`{"credential_id":"%s"}`, credID.String()))

	// First request should pass
	req1 := httptest.NewRequest(http.MethodPost, "/reconcile", bytes.NewReader(body))
	rec1 := httptest.NewRecorder()
	handler.ServeHTTP(rec1, req1)
	if rec1.Code != http.StatusOK {
		t.Errorf("Expected 200 OK for first request, got %d", rec1.Code)
	}

	// Second request immediately should be blocked (rate=10, burst=1)
	req2 := httptest.NewRequest(http.MethodPost, "/reconcile", bytes.NewReader(body))
	rec2 := httptest.NewRecorder()
	handler.ServeHTTP(rec2, req2)
	if rec2.Code != http.StatusTooManyRequests {
		t.Errorf("Expected 429 Too Many Requests, got %d", rec2.Code)
	}

	// Wait for token refill (1/10th second = 100ms)
	time.Sleep(110 * time.Millisecond)

	// Third request should pass again
	req3 := httptest.NewRequest(http.MethodPost, "/reconcile", bytes.NewReader(body))
	rec3 := httptest.NewRecorder()
	handler.ServeHTTP(rec3, req3)
	if rec3.Code != http.StatusOK {
		t.Errorf("Expected 200 OK after sleep, got %d", rec3.Code)
	}
}

func TestRateLimiter_IgnoresInvalidBody(t *testing.T) {
	rl := NewRateLimiter(10, 1) // 10 req/s, burst 1

	handler := rl.Handler(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))

	// Invalid body
	body := []byte(`{"credential_id":"not-a-uuid"}`)

	req := httptest.NewRequest(http.MethodPost, "/reconcile", bytes.NewReader(body))
	rec := httptest.NewRecorder()
	handler.ServeHTTP(rec, req)

	// Should pass through to handler because it's not valid JSON with uuid
	if rec.Code != http.StatusOK {
		t.Errorf("Expected 200 OK, got %d", rec.Code)
	}
}

func TestRateLimiter_IgnoresGET(t *testing.T) {
	rl := NewRateLimiter(10, 1)

	handler := rl.Handler(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))

	req := httptest.NewRequest(http.MethodGet, "/reconcile", nil)
	rec := httptest.NewRecorder()
	handler.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Errorf("Expected 200 OK, got %d", rec.Code)
	}
}
