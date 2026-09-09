<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: design-source/LOADING_AND_FAILURE_STATES.md
Authority: Design source of truth
This file is normative unless explicitly marked as informative in its body.
-->
# Loading, Empty, and Failure States

> **Document role:** Normative UX/visual specification.

Every meaningful asynchronous state must have an explicit UI.

## Loading

Use structural skeletons matching the final content.

Required skeletons:
- transaction list
- transaction detail
- architecture/demo panel
- synchronization status
- data table

Do not use decorative shimmer effects as the primary explanation.

## Empty

An empty state explains:
- what is missing
- why it is empty
- the safe next action

## Network unavailable

Use:
CONNECTIVITY / OFFLINE

Then explain which capabilities remain available.

## Transport unavailable

Example:
NFC is unavailable on this device.

Offer another supported method where appropriate.

## Sync delayed

Use:
SYNCHRONIZATION / WAITING

Avoid framing normal delay as an error.

## Sync failed

Explain:
- what failed
- whether data remains safely stored
- whether retry is safe

## Security failure

Use:
VERIFICATION / FAILED

Explain that the transaction could not be authenticated.

Do not blame the network unless network failure is actually the cause.

## Conflict

Use:
RECONCILIATION / CONFLICT

This state should be visually distinct and should not be auto-dismissed.
