<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: design-source/COMPONENT_LIBRARY.md
Authority: Design source of truth
This file is normative unless explicitly marked as informative in its body.
-->
# Component Library

> **Document role:** Normative UX/visual specification.

## Component inventory

### Navigation
- SiteHeader
- SectionIndex
- DocumentationNav
- Footer

### Status
- ConnectivityState
- TransactionState
- SyncStatus
- SecurityStatus

### Data
- TechnicalNote
- MetricBlock
- DataTable
- ProtocolFieldList
- StateTimeline

### Product
- AmountEntry
- RecipientIdentity
- PaymentMethodSelector
- PaymentConfirmation
- TransactionReceipt
- QRPanel
- NFCPanel
- BLEPanel
- SyncPanel

### Research
- ExperimentCard
- MethodBlock
- ResultBlock
- LimitationBlock
- ArchitectureDiagram

### Loading
- SkeletonTransaction
- SkeletonTable
- SkeletonDiagram
- SyncProgress

## Component requirement

Each component must document:
- purpose
- anatomy
- data inputs
- variants
- states
- accessibility
- empty state
- loading state
- error state
- protocol assumptions
- responsive behavior

## Component rule

Reuse an existing component before creating another variant. A new visual pattern requires documented justification and design review.
