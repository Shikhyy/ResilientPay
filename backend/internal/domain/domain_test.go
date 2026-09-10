// Package domain provides tests for core domain validation.
package domain

import (
	"testing"
	"time"

	"github.com/google/uuid"
)

func TestMoneyValidate(t *testing.T) {
	tests := []struct {
		name    string
		money   Money
		wantErr bool
	}{
		{"valid INR", Money{100, "INR"}, false},
		{"zero amount valid", Money{0, "INR"}, false},
		{"at ceiling", Money{MaxAmountMinor, "INR"}, false},
		{"above ceiling", Money{MaxAmountMinor + 1, "INR"}, true},
		{"empty currency", Money{100, ""}, true},
		{"currency too long", Money{100, "TOOLONGCODE"}, true},
		{"USD valid", Money{500, "USD"}, false},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			err := tc.money.Validate()
			if (err != nil) != tc.wantErr {
				t.Errorf("Validate() err = %v, wantErr %v", err, tc.wantErr)
			}
		})
	}
}

func TestTransactionStateTerminal(t *testing.T) {
	terminal := []TransactionState{StateReconciled, StateRejected, StateConflict}
	nonTerminal := []TransactionState{
		StateCreated, StateValidating, StateAuthorized, StateSigned,
		StateTransferred, StateReceived, StateLocallyVerified, StateLocallyRecorded,
		StateSyncPending,
	}

	for _, s := range terminal {
		if !s.IsTerminal() {
			t.Errorf("expected %s to be terminal", s)
		}
	}
	for _, s := range nonTerminal {
		if s.IsTerminal() {
			t.Errorf("expected %s to be non-terminal", s)
		}
	}
}

func TestCredentialIsActive(t *testing.T) {
	now := time.Now()

	cred := &Credential{
		CredentialID:   uuid.New(),
		SubjectKeyID:   uuid.New(),
		PublicKeyBytes: make([]byte, 32),
		IssuedAt:       now.Add(-1 * time.Hour),
		ExpiresAt:      now.Add(1 * time.Hour),
		State:          CredentialActive,
	}

	if !cred.IsActive(now) {
		t.Error("active unexpired credential must be active")
	}

	// Expired
	cred.ExpiresAt = now.Add(-1 * time.Minute)
	if cred.IsActive(now) {
		t.Error("credential past expiry must not be active")
	}

	// Suspended
	cred.ExpiresAt = now.Add(1 * time.Hour)
	cred.State = CredentialSuspended
	if cred.IsActive(now) {
		t.Error("suspended credential must not be active")
	}
}
