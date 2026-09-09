
# Shared Packages

Shared packages contain artifacts that legitimately need to be consumed by multiple products.

## Intended package groups

### Protocol types

Canonical definitions used across SDK, backend, simulator, and test tooling where cross-language contracts require them.

### Design tokens

Approved visual tokens shared by web and potentially Android implementations.

### Test vectors

Canonical data used to verify serialization, signatures, state handling, and other protocol behavior across implementations.

## Boundary rule

Do not move arbitrary helper functions into `packages/` simply because two components happen to use them. Shared packages must have a clear ownership and compatibility contract.

## Stability

A shared package should have:
- defined API
- tests
- versioning expectations
- owner
- documentation
- compatibility policy

Protocol test vectors deserve particular care because they are an interoperability contract between different language implementations.
