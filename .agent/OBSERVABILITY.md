<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: .agent/OBSERVABILITY.md
Authority: Agent operating procedure
This file is normative unless explicitly marked as informative in its body.
-->
# Agent Observability Contract

> **Document role:** Coding-agent operating procedure.

## Metrics

- `payment_attempts_total`
- `payment_authorized_total`
- `payment_rejected_total`
- `offline_transactions_total`
- `reconciliation_latency_ms`
- `transport_failures_total{transport}`
- `duplicate_transactions_total`
- `signature_failures_total`
- `counter_replay_failures_total`
- `risk_model_score_distribution`

## Logs

Log event IDs, state transitions, error classes, and correlation IDs. Do not log private keys, PINs, raw secrets, or unnecessary sensitive payment data.

## Tracing

Use correlation IDs across mobile/backend/reconciliation paths. Never place secrets in trace attributes.

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

<!-- Deepened -->
## Alerting considerations

For research/demo environments, useful alerts include repeated signature failures from one device, counter anomalies, sudden conflict spikes, transport failure spikes, reconciliation backlog growth, and model-service errors. Alerts should reference aggregate metrics and IDs rather than dumping sensitive payloads.

## Operational correlation

The same `transaction_id` can appear across device, transport, backend, and experiment logs. `correlation_id` identifies a request/trace and may change across retries. This separation makes debugging possible without creating ambiguous identity semantics.
