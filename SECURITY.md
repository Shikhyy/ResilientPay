
# Security Policy

ResilientPay is a research prototype involving payment-like state, cryptographic credentials, local storage, and reconciliation. Security boundaries are therefore treated as first-class design artifacts.

## Reporting

Suspected vulnerabilities should be reported through the project's designated security contact. Do not publish proof-of-concept exploit material before coordinated review.

## Secrets

Never commit:
- private keys
- API secrets
- cloud credentials
- production tokens
- real banking credentials
- real customer personal data

## Security boundary

The following are security-critical:
- credential lifecycle
- signing and verification
- canonical serialization
- counters and replay controls
- local ledger integrity
- reconciliation validation
- authorization policy

The following are not security authorities:
- UI
- transport availability
- ML scoring
- local display state

## Research disclosure

Prototype results must not be represented as production security certification. Independent review, penetration testing, and formal operational controls would be required for any production context.
