-- Migration: 0003_add_settled_state_and_counter_unique.sql
-- Purpose:
-- 1. Add 'SETTLED' to allowed transactions.state values.
-- 2. Add unique constraint on (credential_id, counter) to prevent double-spend replay.

ALTER TABLE transactions DROP CONSTRAINT IF EXISTS transactions_state_check;
ALTER TABLE transactions ADD CONSTRAINT transactions_state_check CHECK (state IN (
    'CREATED', 'VALIDATING', 'AUTHORIZED', 'SIGNED',
    'TRANSFERRED', 'RECEIVED', 'LOCALLY_VERIFIED',
    'LOCALLY_RECORDED', 'SYNC_PENDING',
    'RECONCILED', 'REJECTED', 'CONFLICT', 'SETTLED'
));

-- Replay defense: A given counter for a given credential must never be reused across distinct transactions.
CREATE UNIQUE INDEX IF NOT EXISTS uq_transactions_credential_counter ON transactions (credential_id, counter);
