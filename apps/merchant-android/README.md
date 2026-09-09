
# Merchant Android Application

The merchant application is optimized for receiving and verifying payment transactions in environments where connectivity may be unreliable.

## Responsibilities

The merchant application owns:
- payment request presentation
- QR and proximity interaction surfaces
- receive flow
- local transaction presentation
- operational transaction history
- synchronization status
- conflict presentation
- Android device integration
- accessibility and responsive layouts

## Core rule

The merchant app is not the source of final settlement authority.

A transaction can be:
- locally verified
- locally stored
- awaiting reconciliation
- reconciled
- rejected
- in conflict

These states must remain distinct.

## Operational priorities

The receive experience should minimize unnecessary steps and make the following information immediately legible:
- amount
- payer reference
- local verification status
- transaction reference
- synchronization state

The interface must explain what the merchant can safely treat as received locally versus what has been confirmed by the backend.

See `design/merchant/`, `docs/05-protocol/`, and `docs/04-security/`.
