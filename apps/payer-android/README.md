
# Payer Android Application

The payer application is the customer-facing Android product. It provides the interaction layer over the ResilientPay SDK and the Android-specific capabilities needed to authenticate the user, interact with nearby devices, store local records, and present transaction state.

## Responsibilities

The payer application owns:
- navigation
- screen composition
- user input
- Android lifecycle handling
- device capability discovery
- presentation of connectivity and transaction state
- user-facing error and recovery flows
- accessibility
- analytics that are explicitly approved

## It must not own

The app must not independently implement:
- payment authorization semantics
- signature verification logic
- replay rules
- reconciliation authority
- protocol state definitions
- monetary calculation rules

Those belong to the shared implementation and backend contracts.

## Offline UX

The application must distinguish:
- online processing
- local authorization
- stored transaction
- synchronization pending
- reconciled transaction
- security rejection

The app must never report a locally authorized transaction as finally reconciled unless authoritative reconciliation confirms it.

## Development approach

Build the payer app after the SDK can execute the core transaction lifecycle. Begin with a thin vertical flow over the SDK, then add transports and advanced UX incrementally.

See `design/payer/`, `docs/05-protocol/`, and `docs/06-development/`.
