# M8 Security Review

## Threat Model & Constraints

### 1. Private Key Exposure
**Status: VERIFIED (Host), UNVERIFIED (Android Runtime)**
- **Design:** The SDK design mandates that private key generation and signing strictly occur inside the Android Keystore (`HardwareKeyManager`). The `ResilientPayClient` only passes an opaque `String` alias across the JNI boundary.
- **Concern:** We cannot guarantee absolute StrongBox extraction resistance until M7 is proven on physical hardware, but the API design explicitly forbids exporting bytes.

### 2. Replay Gaps
**Status: ADDRESSED**
- **Design:** The `PaymentEnvelope` mandates a strictly monotonic `counter` bound to the unique `credential_id`. The SDK validator enforces that any incoming counter must be `> last_seen_counter`.

### 3. Duplicate Handling / Conflicting Transactions
**Status: ADDRESSED**
- **Design:** The Go Backend Reconciliation engine enforces idempotency via PostgreSQL unique constraints on `(transaction_id)`. Re-submitting the exact identical bytes yields a safe idempotent 200 OK. Modifying bytes to double-spend yields an invalid signature or a counter replay rejection.

### 4. Unbounded Offline Value
**Status: ADDRESSED**
- **Design:** The Rust `Validator` requires an injected `ValidationContext` containing the known `max_outstanding` budget for the credential. Offline transfers physically cannot exceed this ceiling.

### 5. Money Precision
**Status: VERIFIED**
- **Design:** All money values in Rust, CBOR, Kotlin, and Go use `u64` / `int64` minor units (e.g., cents). Floating point is completely forbidden in the critical path.

### 6. Idempotency
**Status: ADDRESSED**
- **Design:** Network transmissions (Merchant -> Backend) are idempotent. `PaymentEnvelope` generation is state-mutating on the Payer's device, but retrying the *transport* layer does not generate a new signature.

### 7. Backend Trust Assumptions
**Status: INTENTIONALLY DEFERRED**
- **Design:** In M8, the Backend implicitly trusts the Issuer to set the budget correctly. Future milestones will introduce a zero-knowledge or multi-party signature scheme to prevent the Backend from maliciously inflating budgets, but M8 assumes a trusted internal reconciliation engine.
