<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: design-source/USER_FLOWS.md
Authority: Design source of truth
This file is normative unless explicitly marked as informative in its body.
-->
# Core User Flows

> **Document role:** Normative UX/visual specification.

## Flow A: normal online payment

Connected
→ payer enters amount
→ selects payee
→ authenticates
→ transaction created
→ submitted
→ verified
→ reconciled
→ receipt

## Flow B: offline proximity payment

Internet unavailable
→ payment capability checked
→ policy allows bounded offline transaction
→ payer authenticates
→ signed transaction generated
→ NFC/BLE/QR exchange
→ merchant verifies
→ both sides store local record
→ sync later
→ backend reconciliation
→ status updated

## Flow C: SMS-assisted synchronization

Local transaction exists
→ telecom path available
→ compact transaction envelope serialized
→ SMS sent
→ delivery may be delayed/duplicated/reordered
→ backend verifies signature and idempotency
→ reconciliation
→ acknowledgement
→ device updates status

## Flow D: failed security validation

Transaction received
→ signature invalid
→ reject
→ security event recorded
→ no automatic repeated submission
→ show recovery path

## Flow E: duplicate submission

Same transaction ID received again
→ idempotency lookup
→ if identical previously accepted record: return prior result
→ if conflicting content: create reconciliation conflict/security event
→ never silently overwrite authoritative data
