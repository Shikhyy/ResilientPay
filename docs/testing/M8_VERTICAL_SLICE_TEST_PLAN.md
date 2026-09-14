# M8 Vertical Slice Test Plan

This document outlines the end-to-end integration scenarios for the M8 payment slice.

## 1. Happy Path
**Flow:**
1. Payer initiates `createPayment(merchantId, 500)` -> State: `Created`.
2. Rust calls Keystore -> State: `Signed`.
3. SDK pushes to Mock Transport -> State: `Transferred`.
4. Merchant receives CBOR bytes via Mock Transport.
5. Merchant SDK parses and calls `verify()` -> Validation checks pass -> State: `Locally Verified`.
6. Merchant saves to local hash-linked ledger.
7. Merchant syncs ledger to Go Backend.
8. Go Backend persists to PostgreSQL -> State: `Reconciled`.

## 2. Offline Path
**Flow:**
1. Payer disconnected from internet.
2. Payer authorizes within known offline budget -> `Signed` -> `Transferred`.
3. Merchant disconnected from internet.
4. Merchant receives bytes and locally verifies -> `Locally Verified`.
5. Merchant reconnects 4 hours later and pushes ledger payload.
6. Backend verifies signature and persistence -> `Reconciled`.

## 3. Replay Path
**Flow:**
1. Same `PaymentEnvelope` transmitted twice by Payer (or intercepted).
2. Merchant receives first -> `Locally Verified`.
3. Merchant receives second -> `verify()` fails due to strict monotonic counter replay logic.
4. If Merchant mistakenly submits exact duplicate to Backend -> Idempotency key ignores it, returns success state matching original record.

## 4. Tampering Path
**Flow:**
1. Payer signs $5.00 envelope.
2. Malicious transport intercepts, alters CBOR amount to $50.00.
3. Merchant receives altered bytes.
4. Merchant `verify()` immediately rejects due to `Ed25519` signature failure against altered canonical bytes.

## 5. Expiration Path
**Flow:**
1. Envelope generated with strict TTL (e.g., 5 minutes).
2. Payer struggles to transfer for 6 minutes.
3. Merchant receives bytes.
4. Merchant `verify()` strictly checks timestamp against current device clock (within allowable skew) -> Rejects.

## 6. Budget Path
**Flow:**
1. Payer offline budget is $10.00.
2. Payer initiates $15.00 envelope.
3. Payer's Rust SDK rejects pre-signing due to `ValidationContext` offline budget exceeded.
4. UI transitions to `Rejected`.

## 7. Duplicate Backend Submission
**Flow:**
1. Merchant pushes `Locally Verified` ledger to Backend. Network times out, but Backend succeeded.
2. Merchant safely retries identical payload.
3. Backend acknowledges existing state seamlessly (Idempotent success).
