# Backend

> **Research prototype** — this is NOT a production UPI or banking service.

The ResilientPay backend provides server-side reconciliation, credential lifecycle management, and audit logging for the research prototype.

## Architecture

```
cmd/server/main.go
    │
    ├── internal/api/          HTTP API handlers
    ├── internal/reconciliation/  8-step normative ingestion service
    ├── internal/crypto/       Ed25519 verifier + canonical CBOR encoder
    ├── internal/domain/       Core types (Money, states, enums, entities)
    └── internal/store/        Store interface + MemStore (PostgreSQL TBD)
```

## Build and run

```bash
cd backend
go build ./...
go run ./cmd/server/          # listens on :8080 by default
go run ./cmd/server/ --addr :9090   # custom port
RESILIENTPAY_ADDR=:9090 go run ./cmd/server/
```

## Tests

```bash
go test ./...               # all packages
go test ./internal/crypto/  # includes cross-SDK CBOR byte equality test
go test -v ./...            # verbose output
```

**Test count: 40 passing, 0 failing**

| Package | Tests | What's covered |
|---|---|---|
| `internal/domain` | 9 | Money validation, state terminality, credential lifecycle |
| `internal/reconciliation` | 12 | Full 8-step flow: accepted, duplicate, conflict, sig failure, expired/suspended credential, counter/policy limits, schema |
| `internal/api` | 8 | HTTP handler: health, reconcile 201/409/400, transaction 404/200, credential 404 |
| `internal/crypto` | 11 | **Cross-SDK CBOR byte equality**, Ed25519 round-trip, tamper rejection |

### Critical: Cross-SDK test vector

`TestCrossSDKTestVector_CBORMatches` in `internal/crypto` verifies that the Go
canonical encoder produces **bit-for-bit identical** CBOR bytes to the Rust SDK.
The expected bytes are the Gate 3 frozen values from
`sdk/core/tests/protocol_test_vectors.rs`.

If this test fails, **all transaction submissions will fail signature verification**
because the backend will re-derive different canonical bytes than what the device signed.

## API

See [`openapi.yaml`](openapi.yaml) for the full OpenAPI 3.1 specification.

| Method | Path | Result codes |
|---|---|---|
| `GET` | `/v1/health` | 200 |
| `POST` | `/v1/test-credentials` | 201, 400, 500 |
| `POST` | `/v1/reconciliation/transactions` | 201 ACCEPTED, 200 ALREADY_KNOWN, 409 CONFLICT, 422 REJECTED |
| `GET` | `/v1/transactions/{tx_id}` | 200, 404 |
| `GET` | `/v1/credentials/{credential_id}` | 200, 404 |

## Reconciliation flow

Normative 8-step ingestion per `docs/05-protocol/RECONCILIATION_SPEC.md §3`:

1. **Schema validate** — protocol version, nonce length, signature length, nil UUIDs
2. **Authenticate signature** — re-derive canonical CBOR; verify Ed25519 (never trust client)
3. **Check credential** — must be `ACTIVE` and not expired at time of receipt
4. **Idempotency** — exact duplicate → `ALREADY_KNOWN` (no side effect)
5. **Conflict detection** — same tx_id, different content → `CONFLICT` (first-class record)
6. **Counter/policy** — counter ≥ 1, within credential max, amount within per-tx limit
7. **Persist** — save transaction record + emit audit event
8. **Return outcome** — `ACCEPTED` / `ALREADY_KNOWN` / `REJECTED` / `CONFLICT`

## Crypto

| Component | Implementation | Notes |
|---|---|---|
| Ed25519 verify | Go stdlib `crypto/ed25519` | No external dependency |
| CBOR encode | `github.com/fxamacker/cbor/v2` | 13-element array, `SortNone`, `IndefLengthForbidden` |
| Domain separator | `b"resilientpay:payment-envelope:v1:"` | MUST match Rust SDK `SIGNING_DOMAIN_SEPARATOR` |

## Database schema

Migrations are in [`migrations/`](migrations/).

```
0001_initial_schema.sql — credentials, transactions, transaction_events,
                          reconciliation_submissions, audit_events, offline_budgets
```

Apply with:
```bash
psql -U postgres -d resilientpay_dev -f migrations/0001_initial_schema.sql
```

Money is stored as `BIGINT` minor units with a `TEXT` currency column.  
No `NUMERIC`, `FLOAT`, or `DOUBLE PRECISION` columns are used for monetary amounts.  
`BYTEA` columns for keys and signatures include `octet_length()` `CHECK` constraints.

## Security rules

- **Never trust client-reported finality** — all properties re-verified server-side
- **No float for money** — integer minor units throughout
- **No private key material** returned from any API endpoint
- **Conflicts preserved** — never silently overwritten
- **Audit events** emitted for every reconciliation outcome

## Outstanding TODOs

- [ ] PostgreSQL store implementation (replace `MemStore`)
- [ ] PostgreSQL atomic `SaveTransactionWithBudget` via pgx `Tx` (currently falls back to two sequential calls)
- [ ] Database migration runner integration
- [ ] Rate limiting per credential
- [x] mTLS or bearer token authentication — `RESILIENTPAY_ISSUER_SECRET` bearer token guard added to `POST /v1/credentials`
- [x] Offline budget enforcement — `offline_budgets` table enforced; atomic `SaveTransactionWithBudget` eliminates TOCTOU window (MemStore)
