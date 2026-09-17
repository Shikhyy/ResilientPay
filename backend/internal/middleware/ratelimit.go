package middleware

import (
	"bytes"
	"encoding/json"
	"io"
	"net"
	"net/http"
	"sync"
	"time"

	"github.com/google/uuid"
)

// RateLimiter implements a per-credential token bucket rate limiter with an
// IP-address fallback for requests that do not carry a credential_id.
//
// Security design:
//   - Per-credential limit: applied when the request body contains a parseable
//     credential_id (JSON reconciliation submissions).
//   - Per-IP fallback: applied when the body is not JSON or does not contain a
//     credential_id (e.g., SMS webhook text/plain). Prevents unauthenticated
//     callers from flooding non-JSON endpoints without any rate control.
type RateLimiter struct {
	mu          sync.Mutex
	rate        float64
	burst       int
	buckets     map[string]*bucket // keyed by credential UUID string or remote IP
	lastCleanup time.Time
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
		buckets:     make(map[string]*bucket),
		lastCleanup: time.Now(),
	}
}

func (rl *RateLimiter) allow(key string) bool {
	rl.mu.Lock()
	defer rl.mu.Unlock()

	now := time.Now()

	// Periodically cleanup to prevent map from growing unbounded.
	if now.Sub(rl.lastCleanup) > 5*time.Minute {
		rl.cleanup(now)
		rl.lastCleanup = now
	}

	b, exists := rl.buckets[key]
	if !exists {
		rl.buckets[key] = &bucket{
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

// cleanup removes buckets that haven't been accessed recently.
// Must be called with lock held.
func (rl *RateLimiter) cleanup(now time.Time) {
	for id, b := range rl.buckets {
		if now.Sub(b.lastUpdate) > 10*time.Minute {
			delete(rl.buckets, id)
		}
	}
}

// remoteIP extracts the host part of r.RemoteAddr for use as a rate-limit key.
func remoteIP(r *http.Request) string {
	host, _, err := net.SplitHostPort(r.RemoteAddr)
	if err != nil {
		return r.RemoteAddr
	}
	return host
}

// Handler returns an HTTP middleware that enforces rate limits on POST requests.
//
// Rate-limit key selection:
//  1. If the body is JSON and contains a non-nil credential_id, the key is the
//     credential UUID (per-credential limit — primary mechanism).
//  2. Otherwise (non-JSON body, missing or nil credential_id), the key is the
//     remote IP address (fallback — covers SMS webhook and similar endpoints).
func (rl *RateLimiter) Handler(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Only rate-limit POST requests.
		if r.Method != http.MethodPost {
			next.ServeHTTP(w, r)
			return
		}

		// Limit body read to 64 KB to prevent unbounded memory consumption (RP-BK-005).
		// The SMS handler applies its own 8 KB limit on top of this.
		limitedReader := http.MaxBytesReader(w, r.Body, 64*1024)
		body, err := io.ReadAll(limitedReader)
		if err != nil {
			http.Error(w, `{"error":"request body too large or invalid"}`, http.StatusBadRequest)
			return
		}
		r.Body.Close()
		r.Body = io.NopCloser(bytes.NewBuffer(body))

		// Determine rate-limit key.
		rateLimitKey := remoteIP(r) // fallback: IP-based
		var req struct {
			CredentialID uuid.UUID `json:"credential_id"`
		}
		if err := json.Unmarshal(body, &req); err == nil && req.CredentialID != uuid.Nil {
			// Primary: per-credential limit.
			rateLimitKey = req.CredentialID.String()
		}

		if !rl.allow(rateLimitKey) {
			w.Header().Set("Content-Type", "application/json")
			w.WriteHeader(http.StatusTooManyRequests)
			_, _ = w.Write([]byte(`{"error":"rate limit exceeded","code":"RATE_LIMITED"}`))
			return
		}

		next.ServeHTTP(w, r)
	})
}

