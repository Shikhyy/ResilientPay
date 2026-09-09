<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/development/15_RISK_ENGINE.md
Authority: Development contract
This file is normative unless explicitly marked as informative in its body.
-->
# 15 - Risk Engine

> **Document role:** Normative source-of-truth document.

## 1. Principle

ML is an advisory risk estimator. It is not the cryptographic authorization authority and cannot override hard protocol or regulatory constraints.

## 2. Features

Candidate features:

- transaction amount;
- transaction velocity;
- offline duration;
- credential age;
- device history;
- merchant history;
- previous reconciliation failures;
- counter behavior;
- local transaction density;
- network/transport reliability indicators.

Do not use features that create inappropriate privacy, fairness, or compliance risks without explicit research justification and governance.

## 3. Output

A model may return:

```text
risk_score ∈ [0,1]
model_version
feature_schema_version
explanation_code(s)
```

## 4. Decision layering

```text
Hard protocol rules
      ↓
Credential + counter + value checks
      ↓
Optional risk score
      ↓
Policy action
```

The model cannot authorize an otherwise invalid signature or invalid credential.

## 5. Model progression

Start with interpretable baselines:

1. Logistic regression.
2. Decision tree / random forest.
3. Gradient boosting / XGBoost.

Only add more complex models when the experiment demonstrates measurable benefit.

## 6. Evaluation

Use precision, recall, PR-AUC/ROC-AUC as appropriate, calibration, false rejection cost, false acceptance cost, and subgroup analysis where data supports it.

## 7. Deployment

Backend is the authoritative model-development environment. A small model may be exported to an on-device runtime for experiments. Model version must be included in telemetry.

## 8. Safe failure

If the model is missing, corrupt, too old, or unavailable, the system falls back to deterministic rules rather than failing open.

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
