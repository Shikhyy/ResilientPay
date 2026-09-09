<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: .agent/security-rules.md
Authority: Agent operating procedure
This file is normative unless explicitly marked as informative in its body.
-->
# Security Rules

> **Document role:** Coding-agent operating procedure.

These rules are hard constraints.

1. No custom cryptographic primitives.
2. No plaintext private-key storage.
3. No secrets in source control.
4. No secrets or raw credentials in logs.
5. No trust in client-reported settlement/finality.
6. No floating point for money.
7. No timestamp-only replay defense.
8. No insecure TLS bypasses.
9. No authentication bypass in production-like code paths.
10. No UPI PINs/passwords/private keys in SMS.
11. No ML override of hard authorization rules.
12. No silent increase of offline value or credential budget.
13. No direct UI access to secret/key material.
14. No protocol parser that assumes input is honest.
15. Fail closed on cryptographic validation failure.

<!-- Enriched: detailed implementation guidance -->


## Engineering interpretation and implementation notes

### Normative language
The words **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are used intentionally. MUST/MUST NOT define requirements that an implementation cannot change without a specification/ADR update. SHOULD/SHOULD NOT define strong recommendations that may be overridden only with a documented reason.

### State and evidence rule
A user-visible success message, transport callback, database row, or ML result is not independently authoritative. The authoritative meaning of an event comes from the protocol and state machine governing it. Any implementation that shortcuts the defined verification path is a protocol defect even when it makes a demo appear more reliable.

### Failure-first implementation
For every happy-path step, design its failure path before coding it. Ask: what if the process dies immediately before/after this write? What if the message arrives twice? What if the bytes are modified? What if the device clock is wrong? What if the backend response is lost after acceptance? What if a credential is revoked while the device is disconnected? The answer must be represented in state, error semantics, or an explicit documented residual risk.

### Reproducibility
Security-sensitive and research-sensitive behavior must be deterministic where practical. Test vectors, configuration, protocol version, database schema version, model version, and simulator seed are part of the evidence. A screenshot is not sufficient evidence for a security property.

### Change-control trigger
A change to a field, state, key lifecycle, counter rule, offline budget, transport meaning, reconciliation rule, risk boundary, or trust assumption MUST be treated as a specification change and reviewed through `.agent/CHANGE_CONTROL.md`.
