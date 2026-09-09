
# ResilientPay SDK

The SDK is the reusable payment-technology layer of the project. It is the first major
implementation target and the common dependency used by the payer and merchant applications.

## Implementation status

| Module | Status | Notes |
|---|---|---|
| `sdk/core/` (Rust) | ✅ In progress | Domain model, state machine, validation — no crypto yet |
| `sdk/android/` (Kotlin) | 🔲 Not started | Requires Java/Kotlin/Gradle toolchain |
| Canonical serialization (CBOR) | 🔲 Gate 3 | Requires canonical profile decision |
| Cryptographic boundary | 🔲 Gate 3 | Ed25519 — planned in `sdk/core/src/crypto.rs` |
| Local ledger | 🔲 Gate 4 | Depends on serialization |
| Transport contract | 🔲 Gate 5 | After ledger is stable |

## Current: `sdk/core/` (Rust crate `resilientpay-core`)

### Modules

| File | Contents |
|---|---|
| `src/lib.rs` | Module root, re-exports, `PROTOCOL_VERSION` constant |
| `src/money.rs` | `Money` type — integer minor units, explicit currency, checked arithmetic |
| `src/types.rs` | Domain ID newtypes — `TransactionId`, `CredentialId`, `MerchantId`, `KeyId`, `IssuerId`, `MessageId` |
| `src/envelope.rs` | `PaymentEnvelopeCore` — all signed fields per protocol spec; `EnvelopeBuilder` |
| `src/credential.rs` | `OfflineCredential`, `CredentialLifecycleState` |
| `src/state_machine.rs` | `TransactionState`, `TransactionEvent`, `apply_event()` |
| `src/validation.rs` | `Validator`, `ValidationResult`, `ValidationContext` |
| `src/errors.rs` | `ValidationError`, `TransitionError`, `CoreError` |

### Build

```bash
cd sdk/core
cargo build
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

### Test coverage

- 83 unit tests + 1 doc-test: **84 total, all passing**
- Covers: all valid state transitions, all invalid transitions, Money arithmetic,
  envelope field validation, credential lifecycle, counter/replay rules, value policy,
  merchant checks, expiry

## Responsibility

The SDK owns the reusable domain and security-adjacent mechanics that should not be duplicated
inside product applications:

- payment domain models
- transaction state
- validation
- protocol envelope construction and parsing
- policy evaluation
- credential interfaces
- cryptographic interfaces and approved implementations _(planned)_
- replay and counter controls
- local ledger abstractions _(planned)_
- transport-independent APIs _(planned)_

## Non-responsibilities

The SDK does not own:
- UI composition
- navigation
- application copy
- Android screen state outside the domain model
- network routing
- merchant presentation
- final settlement claims
- research dashboards

Transport implementations must remain adapters around the protocol.

## Development order

1. ✅ domain model
2. ✅ state machine
3. ✅ validation
4. 🔲 serialization (canonical CBOR — Gate 3)
5. 🔲 cryptographic boundary (Gate 3)
6. 🔲 credential issuing/verifying (Gate 3)
7. 🔲 local ledger (Gate 4)
8. 🔲 transport contract (Gate 5)
9. 🔲 Android adapter (Gate 5)

## Quality requirement

The SDK must be runnable and testable without an Android device for the core transaction path.
Critical behavior must be deterministic and covered by unit, contract, negative, and
property-based tests where appropriate.

## Versioning

SDK version and protocol version are separate. A change that modifies protocol interoperability
requires explicit compatibility analysis even when the public programming API looks unchanged.

See `docs/03-architecture/`, `docs/04-security/`, `docs/05-protocol/`, and `docs/06-development/`.
