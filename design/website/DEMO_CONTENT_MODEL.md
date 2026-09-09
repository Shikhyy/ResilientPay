
# Demo Content Model

## Purpose

The live demonstration is the primary proof mechanism on the website.

## Demo entities

### Device
- device role
- display name
- connectivity capabilities
- current connectivity state

### Payment
- amount
- currency
- merchant reference
- transaction reference
- current state
- transport
- synchronization state
- reconciliation state

### Scenario
- baseline
- connectivity failure
- local transport
- delayed sync
- replay attempt
- tampered payload
- duplicate submission

## Truthfulness

All demo data must be explicitly simulated or drawn from a controlled local environment. Do not present simulated traffic as production activity.

## Interaction

Changing a scenario must change the actual displayed state consistently.

Example:
turning internet off must affect the available path and the transaction state. It must not merely recolor a status indicator.

## Reset

The user must be able to reset the scenario deterministically to a known baseline.
