// Package main is the entrypoint for the ResilientPay backend server.
//
// This is a prototype backend for research purposes.
// It does NOT represent a production UPI or banking service.
//
// Usage:
//
//	./server [--addr :8080]
//
// The server starts on :8080 by default. Set RESILIENTPAY_ADDR to override.
// All protocol parameters (version, key sizes, etc.) are hardcoded per the
// research specification and must not be changed without a change-control review.
package main

import (
	"context"
	"errors"
	"flag"
	"log/slog"
	"net"
	"net/http"
	"os"
	"os/signal"
	"strconv"
	"syscall"
	"time"

	"github.com/Shikhyy/ResilientPay/backend/internal/api"
	bkCrypto "github.com/Shikhyy/ResilientPay/backend/internal/crypto"
	"github.com/Shikhyy/ResilientPay/backend/internal/middleware"
	"github.com/Shikhyy/ResilientPay/backend/internal/reconciliation"
	"github.com/Shikhyy/ResilientPay/backend/internal/settlement"
	"github.com/Shikhyy/ResilientPay/backend/internal/store"
)

func main() {
	addr := flag.String("addr", envOrDefault("RESILIENTPAY_ADDR", ":8080"), "listen address")
	flag.Parse()

	logger := slog.New(slog.NewJSONHandler(os.Stdout, &slog.HandlerOptions{
		Level: slog.LevelInfo,
	}))
	slog.SetDefault(logger)

	slog.Info("starting ResilientPay backend (research prototype)",
		"addr", *addr,
		"note", "this is NOT a production UPI or banking service",
	)

	// Wiring: storage backend (PostgreSQL if DATABASE_URL is set, otherwise in-memory).
	var st store.Store
	dbURL := envOrDefault("DATABASE_URL", "")
	if dbURL != "" {
		pgStore, err := store.NewPostgresStore(context.Background(), dbURL)
		if err != nil {
			slog.Error("failed to connect to postgresql database", "err", err)
			os.Exit(1)
		}
		defer pgStore.Close()
		st = pgStore
		slog.Info("connected to PostgreSQL store", "url_redacted", "configured")
	} else {
		st = store.NewMemStore()
		slog.Info("using in-memory store (DATABASE_URL not set)")
	}

	// Wiring: real Ed25519 verifier (stdlib crypto/ed25519, no external dependency).
	verifier := bkCrypto.NewEd25519Verifier()

	// Wiring: real CBOR canonical encoder — produces identical bytes to the Rust SDK.
	// Cross-SDK byte equality is verified by TestCrossSDKTestVector_CBORMatches.
	encoder := bkCrypto.NewCanonicalEncoder()

	svc := reconciliation.NewService(st, verifier, encoder)

	// SECURITY: RESILIENTPAY_ISSUER_SECRET guards POST /v1/credentials (credential issuance).
	// This MUST be set to a strong random value in any non-local deployment.
	// If unset, credential issuance requires no authentication (dev/test mode only).
	issuerSecret := os.Getenv("RESILIENTPAY_ISSUER_SECRET")
	if issuerSecret == "" {
		slog.Warn("RESILIENTPAY_ISSUER_SECRET is not set; credential issuance is unauthenticated (dev mode only)")
	}

	handler := api.NewHandler(svc, st, issuerSecret)

	mux := http.NewServeMux()
	rateLimitRps := envIntOrDefault("RESILIENTPAY_RATE_LIMIT_RPS", 10)
	rateLimitBurst := envIntOrDefault("RESILIENTPAY_RATE_LIMIT_BURST", 20)
	rl := middleware.NewRateLimiter(float64(rateLimitRps), rateLimitBurst)

	handler.RegisterRoutes(mux)

	// Wiring: Settlement Worker
	settlementIntervalSec := envIntOrDefault("RESILIENTPAY_SETTLEMENT_INTERVAL_SEC", 5)
	settlementBatchSize := envIntOrDefault("RESILIENTPAY_SETTLEMENT_BATCH_SIZE", 100)
	settlementWorker := settlement.NewWorker(st, time.Duration(settlementIntervalSec)*time.Second, settlementBatchSize, logger)
	workerCtx, workerCancel := context.WithCancel(context.Background())
	go settlementWorker.Start(workerCtx)

	srv := &http.Server{
		Addr:         *addr,
		Handler:      rl.Handler(mux),
		ReadTimeout:  10 * time.Second,
		WriteTimeout: 30 * time.Second,
		IdleTimeout:  60 * time.Second,
		BaseContext: func(_ net.Listener) context.Context {
			return context.Background()
		},
	}

	// Graceful shutdown on SIGINT / SIGTERM.
	idleConnsClosed := make(chan struct{})
	go func() {
		sigCh := make(chan os.Signal, 1)
		signal.Notify(sigCh, syscall.SIGINT, syscall.SIGTERM)
		<-sigCh
		slog.Info("shutdown signal received, draining connections")

		ctx, cancel := context.WithTimeout(context.Background(), 15*time.Second)
		defer cancel()
		if err := srv.Shutdown(ctx); err != nil {
			slog.Error("shutdown error", "err", err)
		}
		workerCancel() // stop settlement worker
		close(idleConnsClosed)
	}()

	slog.Info("server listening", "addr", *addr)
	if err := srv.ListenAndServe(); !errors.Is(err, http.ErrServerClosed) {
		slog.Error("ListenAndServe failed", "err", err)
		os.Exit(1)
	}

	<-idleConnsClosed
	slog.Info("server stopped cleanly")
}

// ensure reconciliation interfaces are satisfied at compile time
var _ reconciliation.Verifier = (*bkCrypto.Ed25519Verifier)(nil)
var _ reconciliation.CanonicalEncoder = (*bkCrypto.CanonicalEncoder)(nil)

func envOrDefault(key, def string) string {
	if v := os.Getenv(key); v != "" {
		return v
	}
	return def
}

func envIntOrDefault(key string, def int) int {
	if v := os.Getenv(key); v != "" {
		if i, err := strconv.Atoi(v); err == nil {
			return i
		}
	}
	return def
}
