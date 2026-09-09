<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/development/MONEY.md
Authority: Development contract
This file is normative unless explicitly marked as informative in its body.
-->
# Money Representation

> **Document role:** Normative source-of-truth document.

## Rule

Never use floating-point values to represent money.

## Representation

```text
amount_minor: signed/unsigned integer as appropriate
currency: ISO currency code string
```

For INR:

```text
₹1 = 100 paise
₹99.99 → amount_minor = 9999
```

## Validation

- no negative values where the transaction type forbids them;
- no overflow during arithmetic;
- currency must be explicit;
- serialization must preserve exact integer value;
- formatting for UI occurs only at the presentation boundary.

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

## Canonical parsing

User-entered amounts are presentation strings, not protocol amounts. Parse them through a single locale-aware input layer, convert to integer minor units using explicitly tested rounding rules, then pass only the integer representation into domain logic. The signed protocol must contain the integer representation.
