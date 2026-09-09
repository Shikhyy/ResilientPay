
# Agent Project Manifest

```yaml
project:
  name: resilientpay
  status: research-prototype
  architecture: monorepo

workstreams:
  - sdk
  - payer-android
  - merchant-android
  - backend
  - simulator
  - website
  - design
  - research

core_principles:
  - protocol-first
  - sdk-first
  - transport-independent
  - cryptographically-authenticated
  - bounded-risk
  - deterministic-state
  - test-first
  - evidence-driven

release_model:
  branch: main
  feature_branches: short-lived
  commits: conventional
  merge: human-reviewed

forbidden_shortcuts:
  - invent-cryptography
  - bypass-validation
  - client-authoritative-settlement
  - floating-point-money
  - unreviewed-protocol-change
```

This manifest is descriptive. The detailed Markdown procedures in `.agent/` remain authoritative for agent behavior.
