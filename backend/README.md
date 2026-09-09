
# Backend

The backend provides the authoritative server-side processing required to receive synchronization batches, validate transactions independently, perform idempotent reconciliation, manage credential lifecycle, record audit events, and expose controlled research APIs.

## Initial architecture

Start as a modular service rather than prematurely splitting into many deployable microservices.

Expected logical areas:
- API
- reconciliation
- credential
- audit
- risk
- persistence

## Backend principles

The backend does not trust client-reported final states.

Every submitted transaction should be checked according to the protocol contract before becoming authoritative.

Idempotency is mandatory for retryable synchronization. The same transaction submitted multiple times must not create multiple authoritative records.

Conflicting submissions must not silently overwrite each other.

## Data discipline

Money is stored in integer minor units with explicit currency. Transaction IDs and protocol counters require appropriate uniqueness constraints.

Database migrations are versioned and tested.

See `docs/05-protocol/RECONCILIATION_SPEC.md`, `docs/06-development/DATABASE_SCHEMA.md`, and `docs/06-development/API_SPEC.md`.
