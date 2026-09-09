<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/research/24_SOURCE_REGISTER.md
Authority: Research specification
This file is normative unless explicitly marked as informative in its body.
-->
# 24 - Source Register

> **Document role:** Normative source-of-truth document.

## Official sources checked on 2026-09-08

### RBI - Offline payment framework

Title: Framework for Facilitating Small Value Digital Payments in Offline Mode

URL: https://www.rbi.org.in/scripts/RTGS_Notification.aspx?Id=12215

Use for: definition of offline payment, proximity requirement, AFA, limits, replenishment, alerts, responsibilities, grievance/redressal.

### RBI - PSS Act FAQ

URL: https://www.rbi.org.in/CommonPerson/english/scripts/FAQs.aspx?Id=420

Use for: authorization boundary and payment-system regulatory context.

### RBI - Payment Systems

URL: https://www.rbi.org.in/scripts/paymentsystems.aspx

Use for: official payment-system regulatory information and permitted activities.

### NPCI - UPI Lite

URL: https://www.npci.org.in/product/upi/upi-lite

Use for: UPI Lite product description and current product-level context.

### RBI - Annual Report payments discussion

URL: https://rbi.org.in/scripts/PublicationsView.aspx?id=22459

Use for: historical/current-context description of UPI Lite X, UPI 123PAY, and offline NFC evolution.

## Source policy

Prefer primary RBI/NPCI documents for regulatory/product claims. Secondary sources may be used for ecosystem reporting or industry context but must not override primary sources.

## Citation policy for the paper

Use the official documents above for claims about existing Indian payment mechanisms. Capture access/verification date in the bibliography for reproducibility.

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
