<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: design-source/DEMO_SPEC.md
Authority: Design source of truth
This file is normative unless explicitly marked as informative in its body.
-->
# Landing Page Product Demonstration Specification

> **Document role:** Normative UX/visual specification.

## Purpose

The demonstration is the central proof mechanism on the landing page.

## Demo model

A deterministic simulation should control:
- network state
- transport availability
- transaction amount
- payer device state
- merchant device state
- reconciliation state
- attack scenarios

## Required scenarios

### Scenario 1
Internet available.

Expected:
normal submission path.

### Scenario 2
Internet unavailable, NFC available.

Expected:
local proximity exchange.

### Scenario 3
Internet unavailable, BLE available.

Expected:
local proximity exchange through BLE.

### Scenario 4
Internet unavailable, QR available.

Expected:
optical transfer path.

### Scenario 5
Local payment completed, no network.

Expected:
stored locally, synchronization pending.

### Scenario 6
Connectivity restored.

Expected:
batch submitted, verified, reconciled.

### Scenario 7
Replay attack.

Expected:
replay detected and rejected.

### Scenario 8
Modified amount.

Expected:
signature verification fails.

## UI requirements

The demo must always show:
- current connectivity state
- selected transport
- current transaction state
- reconciliation state

The visitor should be able to pause and inspect each transition.

No fake data should be presented as production activity. Simulation data must be clearly labeled.
