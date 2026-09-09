
# ResilientPay Simulator

The simulator is a controlled environment for exercising the protocol and reconciliation behavior under conditions that are difficult to reproduce manually.

## Objectives

It should model:
- payer
- merchant
- backend
- transport
- credential state
- connectivity state
- time
- message duplication
- message loss
- message reordering
- message delay
- device restarts
- malicious modifications
- replay attempts
- offline duration

## Determinism

A scenario must be reproducible from a versioned configuration and seed when randomness is used.

A simulation result should identify:
- code version
- protocol version
- scenario version
- configuration
- seed
- run identifier

## Research use

The simulator is not merely a demo. It is a source of evidence for reliability and security experiments. Scenario definitions should be reviewed and retained so results can be reproduced later.

See `docs/07-research-experiments/` and `docs/06-development/EXPERIMENT_DATA.md`.
