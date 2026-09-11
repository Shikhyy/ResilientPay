package api

import (
	"errors"

	"github.com/google/uuid"
	"github.com/Shikhyy/ResilientPay/backend/internal/domain"
)

// IssueCredentialRequest defines the body for POST /v1/credentials.
type IssueCredentialRequest struct {
	SubjectKeyIDHex     string `json:"subject_key_id_hex"`
	PublicKeyHex        string `json:"public_key_hex"`
	MaxValuePerTxMinor  uint64 `json:"max_value_per_tx_minor"`
	MaxValueOutstanding uint64 `json:"max_value_outstanding_minor"`
	MaxCounter          uint64 `json:"max_counter"`
	ValidForSeconds     int64  `json:"valid_for_seconds"`
}

// Validate validates the issuance request schema.
func (req *IssueCredentialRequest) Validate() error {
	if len(req.SubjectKeyIDHex) != 32 {
		return errors.New("subject_key_id_hex must be 32 hex chars (16 bytes UUID)")
	}
	if len(req.PublicKeyHex) != 64 {
		return errors.New("public_key_hex must be 64 hex chars (32 bytes Ed25519)")
	}
	if req.MaxValuePerTxMinor == 0 {
		return errors.New("max_value_per_tx_minor must be > 0")
	}
	if req.MaxValueOutstanding < req.MaxValuePerTxMinor {
		return errors.New("max_value_outstanding_minor must be >= max_value_per_tx_minor")
	}
	if req.MaxCounter == 0 || req.MaxCounter > 10000 {
		return errors.New("max_counter must be between 1 and 10000")
	}
	if req.ValidForSeconds < 3600 || req.ValidForSeconds > 31536000 { // 1 hr to 1 year
		return errors.New("valid_for_seconds must be between 1 hour and 1 year")
	}
	return nil
}

// ValidateSubmission validates the reconciliation submission schema.
func ValidateSubmission(sub *domain.TransactionSubmission) error {
	if sub.ProtocolVersion != 1 {
		return errors.New("unsupported protocol_version")
	}
	if sub.TxID == uuid.Nil {
		return errors.New("missing tx_id")
	}
	if sub.CredentialID == uuid.Nil {
		return errors.New("missing credential_id")
	}
	if sub.PayerKeyID == uuid.Nil {
		return errors.New("missing payer_key_id")
	}
	if sub.MerchantID == uuid.Nil {
		return errors.New("missing merchant_id")
	}
	if sub.AmountMinor == 0 {
		return errors.New("amount_minor must be > 0")
	}
	if sub.Currency == "" {
		return errors.New("missing currency")
	}
	if sub.Counter == 0 {
		return errors.New("counter must be > 0")
	}
	if len(sub.Nonce) != 16 {
		return errors.New("nonce must be 16 bytes")
	}
	if len(sub.SignatureBytes) != 64 {
		return errors.New("signature_bytes must be 64 bytes")
	}
	return nil
}
