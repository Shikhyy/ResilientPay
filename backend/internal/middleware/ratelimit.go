package middleware

import (
	"bytes"
	"encoding/json"
	"io"
	"net/http"
	"sync"
	"time"

	"github.com/google/uuid"
)

// RateLimiter implements a per-credential token bucket rate limiter.
type RateLimiter struct {
	mu           sync.Mutex
	rate         float64
	burst        int
	buckets      map[uuid.UUID]*bucket
	lastCleanup  time.Time
}

type bucket struct {
	tokens     float64
	lastUpdate time.Time
}

// NewRateLimiter creates a new RateLimiter.
// rate is tokens per second, burst is the maximum bucket capacity.
func NewRateLimiter(rate float64, burst int) *RateLimiter {
	return &RateLimiter{
		rate:        rate,
		burst:       burst,
		buckets:     make(map[uuid.UUID]*bucket),
		lastCleanup: time.Now(),
	}
}

func (rl *RateLimiter) allow(credID uuid.UUID) bool {
	rl.mu.Lock()
	defer rl.mu.Unlock()

	now := time.Now()
	
	// Periodically cleanup to prevent map from growing unbounded
	if now.Sub(rl.lastCleanup) > 5*time.Minute {
		rl.cleanup(now)
		rl.lastCleanup = now
	}

	b, exists := rl.buckets[credID]
	if !exists {
		rl.buckets[credID] = &bucket{
			tokens:     float64(rl.burst) - 1.0,
			lastUpdate: now,
		}
		return true
	}

	elapsed := now.Sub(b.lastUpdate).Seconds()
	b.tokens += elapsed * rl.rate
	if b.tokens > float64(rl.burst) {
		b.tokens = float64(rl.burst)
	}
	b.lastUpdate = now

	if b.tokens >= 1.0 {
		b.tokens -= 1.0
		return true
	}
	return false
}

// cleanup removes buckets that haven't been accessed recently
// Must be called with lock held
func (rl *RateLimiter) cleanup(now time.Time) {
	for id, b := range rl.buckets {
		if now.Sub(b.lastUpdate) > 10*time.Minute {
			delete(rl.buckets, id)
		}
	}
}

// Handler returns an HTTP middleware handler that enforces rate limits based on credential_id.
func (rl *RateLimiter) Handler(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Only limit POST requests (reconciliation submissions)
		if r.Method != http.MethodPost {
			next.ServeHTTP(w, r)
			return
		}

		// We need to read the body to extract credential_id, and then restore it.
		body, err := io.ReadAll(r.Body)
		if err != nil {
			http.Error(w, `{"error":"bad request"}`, http.StatusBadRequest)
			return
		}
		r.Body.Close()
		r.Body = io.NopCloser(bytes.NewBuffer(body))

		var req struct {
			CredentialID uuid.UUID `json:"credential_id"`
		}
		
		// If it's valid JSON and contains a credential ID, we rate limit it.
		// If it's invalid, we let the actual handler reject it (schema validation step).
		if err := json.Unmarshal(body, &req); err == nil && req.CredentialID != uuid.Nil {
			if !rl.allow(req.CredentialID) {
				w.Header().Set("Content-Type", "application/json")
				w.WriteHeader(http.StatusTooManyRequests)
				w.Write([]byte(`{"error":"rate limit exceeded"}`))
				return
			}
		}

		next.ServeHTTP(w, r)
	})
}
