
# Tests

Testing is distributed across source modules and cross-component suites.

## Test classes

### Unit
Fast verification of deterministic components.

### Contract
Verification that implementations match protocol/API contracts.

### Integration
Verification across module boundaries such as SDK to ledger or backend to database.

### Security
Negative tests for tampering, replay, invalid credentials, malformed data, and authorization boundary failures.

### End-to-end
Payer to merchant to reconciliation flows.

### Device
Actual Android behavior for NFC, BLE, QR, persistence, lifecycle, and platform security.

### Visual and accessibility
Screenshot, layout, focus, semantics, and state coverage for UI.

## Test philosophy

A passing test suite is evidence of tested behavior, not evidence that the design itself is correct. Tests must trace back to requirements and invariants.

Tests should fail clearly when a security property is violated.
