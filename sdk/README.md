
# ResilientPay SDK

The SDK is the reusable payment-technology layer of the project. It is the first major implementation target and the common dependency used by the payer and merchant applications.

## Responsibility

The SDK owns the reusable domain and security-adjacent mechanics that should not be duplicated inside product applications:

- payment domain models
- transaction state
- validation
- protocol envelope construction and parsing
- policy evaluation
- credential interfaces
- cryptographic interfaces and approved implementations
- replay and counter controls
- local ledger abstractions
- transport-independent APIs

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

1. domain model
2. state machine
3. serialization
4. validation
5. cryptographic boundary
6. credentials
7. local ledger
8. transport contract
9. Android adapter

## Quality requirement

The SDK must be runnable and testable without an Android device for the core transaction path. Critical behavior should be deterministic and covered by unit, contract, negative, and property-based tests where appropriate.

## Versioning

SDK version and protocol version are separate. A change that modifies protocol interoperability requires explicit compatibility analysis even when the public programming API looks unchanged.

See `docs/03-architecture/`, `docs/04-security/`, `docs/05-protocol/`, and `docs/06-development/`.
