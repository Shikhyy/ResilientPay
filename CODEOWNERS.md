
# CODEOWNERS Policy

This document describes intended ownership boundaries. The eventual GitHub `CODEOWNERS` file should be generated or maintained from these responsibilities.

## Ownership

### Core protocol and security
- `docs/05-protocol/`
- `docs/04-security/`
- `sdk/core/`
- `sdk/crypto/`
- `packages/test-vectors/`

Require protocol/security owner review.

### Backend
- `backend/`
- `docs/06-development/`
- `docs/08-operations/`

Require backend owner review.

### Android
- `apps/payer-android/`
- `apps/merchant-android/`
- `sdk/android/`

Require Android owner review.

### Design
- `design/`
- `website/`

Require product/design review for meaningful UX changes.

## Final authority

Cross-cutting changes involving security, protocol, trust, regulatory scope, or research methodology require human architectural review even when an automated code owner approves the file.
