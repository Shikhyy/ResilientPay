-- Migration: 0001_initial_schema.sql
-- ResilientPay backend — initial PostgreSQL schema.
--
-- Schema design rules (DATABASE_SCHEMA.md §3):
--   - Unique transaction IDs (UUID primary keys)
--   - Non-negative monetary amounts enforced by CHECK constraints
--   - Integer minor units only (no numeric/float money columns)
--   - Explicit state enums as text with CHECK constraints
--   - Timestamps always timezone-aware (TIMESTAMPTZ)
--   - Backend generates its own event timestamps (not trusted from client)
--   - Idempotency keys on reconciliation submissions
--   - Indexes on transaction_id, credential_id for lookup performance
--
-- Versioning: migrations are numbered, never modified after deployment.
-- Apply with: psql -f 0001_initial_schema.sql or a migration tool.

BEGIN;

-- ---------------------------------------------------------------------------
-- Enums (as constrained TEXT columns for portability)
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS schema_migrations (
    version     TEXT PRIMARY KEY,
    applied_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO schema_migrations (version) VALUES ('0001') ON CONFLICT DO NOTHING;

-- ---------------------------------------------------------------------------
-- credentials
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS credentials (
    credential_id           UUID        PRIMARY KEY,
    subject_key_id          UUID        NOT NULL,
    -- Ed25519 public key — 32 bytes stored as bytea.
    -- Never store private key material here.
    public_key_bytes        BYTEA       NOT NULL CHECK (octet_length(public_key_bytes) = 32),
    issued_at               TIMESTAMPTZ NOT NULL,
    expires_at              TIMESTAMPTZ NOT NULL,
    -- Policy limits in minor units (INR paise). No float.
    max_value_per_tx_minor  BIGINT      NOT NULL CHECK (max_value_per_tx_minor >= 0),
    max_value_outstanding_minor BIGINT  NOT NULL CHECK (max_value_outstanding_minor >= 0),
    max_counter             BIGINT      NOT NULL CHECK (max_counter > 0),
    -- Lifecycle state
    state                   TEXT        NOT NULL CHECK (state IN (
                                'REQUESTED', 'ISSUED', 'ACTIVE', 'SUSPENDED', 'REVOKED', 'EXPIRED'
                            )),
    policy_version          INTEGER     NOT NULL DEFAULT 1,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_credentials_state ON credentials (state);

-- ---------------------------------------------------------------------------
-- transactions
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS transactions (
    -- Protocol fields (from the signed envelope — never modified after insert)
    tx_id                   UUID        PRIMARY KEY,
    credential_id           UUID        NOT NULL REFERENCES credentials (credential_id),
    payer_key_id            UUID        NOT NULL,
    merchant_id             UUID        NOT NULL,
    -- Money in integer minor units only
    amount_minor            BIGINT      NOT NULL CHECK (amount_minor >= 0),
    currency                TEXT        NOT NULL CHECK (char_length(currency) BETWEEN 1 AND 8),
    counter                 BIGINT      NOT NULL CHECK (counter > 0),
    nonce                   BYTEA       NOT NULL CHECK (octet_length(nonce) = 16),
    created_at_unix         BIGINT      NOT NULL,
    expires_at_unix         BIGINT      NOT NULL,
    previous_event_hash     BYTEA       CHECK (previous_event_hash IS NULL OR octet_length(previous_event_hash) = 32),
    risk_class              TEXT,
    -- Ed25519 signature — 64 bytes
    signature_bytes         BYTEA       NOT NULL CHECK (octet_length(signature_bytes) = 64),
    protocol_version        INTEGER     NOT NULL,
    -- Backend-assigned state
    state                   TEXT        NOT NULL CHECK (state IN (
                                'CREATED', 'VALIDATING', 'AUTHORIZED', 'SIGNED',
                                'TRANSFERRED', 'RECEIVED', 'LOCALLY_VERIFIED',
                                'LOCALLY_RECORDED', 'SYNC_PENDING',
                                'RECONCILED', 'REJECTED', 'CONFLICT'
                            )),
    -- Backend timestamps
    backend_received_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_transactions_credential_id ON transactions (credential_id);
CREATE INDEX IF NOT EXISTS idx_transactions_state ON transactions (state);
CREATE INDEX IF NOT EXISTS idx_transactions_merchant_id ON transactions (merchant_id);

-- ---------------------------------------------------------------------------
-- transaction_events (immutable append-only event log)
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS transaction_events (
    event_id        UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    tx_id           UUID        NOT NULL REFERENCES transactions (tx_id),
    event_type      TEXT        NOT NULL,
    from_state      TEXT        NOT NULL,
    to_state        TEXT        NOT NULL,
    -- Hash chain fields (from the local ledger model)
    payload_hash    BYTEA       CHECK (payload_hash IS NULL OR octet_length(payload_hash) = 32),
    previous_chain_hash BYTEA   CHECK (previous_chain_hash IS NULL OR octet_length(previous_chain_hash) = 32),
    chain_hash      BYTEA       CHECK (chain_hash IS NULL OR octet_length(chain_hash) = 32),
    -- Backend-assigned timestamp — not trusted from client
    occurred_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    detail          TEXT
);

CREATE INDEX IF NOT EXISTS idx_tx_events_tx_id ON transaction_events (tx_id);
CREATE INDEX IF NOT EXISTS idx_tx_events_occurred_at ON transaction_events (occurred_at);

-- ---------------------------------------------------------------------------
-- reconciliation_submissions (idempotency log)
-- ---------------------------------------------------------------------------
-- Records every unique submission attempt for idempotency tracking.
-- The idempotency_key is the SHA-256 of the canonical signing input bytes.

CREATE TABLE IF NOT EXISTS reconciliation_submissions (
    submission_id       UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    tx_id               UUID        NOT NULL,
    -- SHA-256 of the canonical bytes submitted — idempotency key
    canonical_hash      BYTEA       NOT NULL CHECK (octet_length(canonical_hash) = 32),
    result              TEXT        NOT NULL CHECK (result IN (
                            'ACCEPTED', 'ALREADY_KNOWN', 'REJECTED', 'CONFLICT'
                        )),
    reason              TEXT,
    submitted_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (canonical_hash)
);

CREATE INDEX IF NOT EXISTS idx_recon_subs_tx_id ON reconciliation_submissions (tx_id);

-- ---------------------------------------------------------------------------
-- audit_events (security and operational audit log — immutable)
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS audit_events (
    event_id        UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    tx_id           UUID,       -- nullable: some events are not per-transaction
    credential_id   UUID,       -- nullable
    kind            TEXT        NOT NULL,
    detail          TEXT,
    occurred_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_audit_tx_id ON audit_events (tx_id);
CREATE INDEX IF NOT EXISTS idx_audit_kind ON audit_events (kind);
CREATE INDEX IF NOT EXISTS idx_audit_occurred_at ON audit_events (occurred_at);

-- ---------------------------------------------------------------------------
-- offline_budgets (tracks cumulative offline spend per credential)
-- ---------------------------------------------------------------------------
-- Used for enforcement of max_value_outstanding policy.
-- updated atomically with transaction acceptance.

CREATE TABLE IF NOT EXISTS offline_budgets (
    credential_id           UUID    PRIMARY KEY REFERENCES credentials (credential_id),
    outstanding_minor       BIGINT  NOT NULL DEFAULT 0 CHECK (outstanding_minor >= 0),
    last_updated_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMIT;
