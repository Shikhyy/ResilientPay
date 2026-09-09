<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: docs/research/01_REGULATORY_LANDSCAPE.md
Authority: Research specification
This file is normative unless explicitly marked as informative in its body.
-->
# 01 - Regulatory Landscape

> **Document role:** Normative source-of-truth document.

> **Status:** Reference guide for research; not legal advice.

## 1. Current reference position

The Reserve Bank of India (RBI) has a framework for small-value digital payments in offline mode. The official framework defines an offline payment as a transaction that does not require Internet or telecom connectivity to take effect. It states that offline payments are to be made in proximity/face-to-face mode, may be offered without mandatory Additional Factor of Authentication (AFA), and are subject to value controls and other participant obligations.

The current RBI framework page states, as updated on 2024-12-04, a general offline upper transaction limit of ₹500 and a total offline limit of ₹2,000 on a payment instrument, with enhanced UPI Lite limits of ₹1,000 per transaction and ₹5,000 total. Replenishment of used limit is specified as online with AFA. These figures are regulatory/product context, **not prototype defaults**; the software must treat limits as configurable policy data.

RBI also notes that customer protection, alerts, acquirer/issuer responsibilities, grievance redressal, and RBI supervisory powers apply to qualifying solutions.

## 2. Existing UPI-related mechanisms

NPCI describes UPI Lite as a low-value, PIN-less payment solution operating using existing UPI ecosystem protocols. RBI publications identify UPI Lite X as an NFC-based offline capability and UPI 123PAY as a feature-phone route.

Therefore the research contribution must not be framed as “inventing offline UPI.” The contribution is the **multi-modal resilience layer, formal protocol model, reconciliation behavior, security analysis, and empirical evaluation**.

## 3. Authorization boundary

RBI states that, under the Payment and Settlement Systems Act, 2007, a person may not operate a payment system in India except under authorization from RBI, subject to the Act and applicable permissions.

Accordingly, this research prototype:

- does not operate a payment system;
- does not represent itself as a UPI participant;
- does not settle real funds;
- does not collect real bank credentials for payment use;
- should be demonstrated with synthetic accounts/transactions unless an authorized institution explicitly provides a lawful sandbox or test environment.

## 4. Regulatory assumptions to avoid

Do not assume that:

- an Android prototype is automatically permitted for production payments;
- copying a UPI-style flow makes the system legally equivalent to UPI;
- SMS delivery is equivalent to an offline payment under RBI's definition;
- an offline protocol is allowed to exceed applicable limits because a local device enforces a limit;
- customer liability or dispute obligations disappear during offline operation;
- an ML risk score can override regulatory or scheme controls.

## 5. Research terminology

Preferred wording:

- connectivity-resilient payments
- low-connectivity payment protocol
- offline-first transaction protocol
- multi-modal transport
- store-and-forward reconciliation
- bounded offline authorization
- eventual reconciliation

Avoid claiming:

- “new UPI”
- “replacement for UPI”
- “guaranteed offline settlement”
- “SMS is offline payment”

## 6. Reference sources

- RBI - Framework for Facilitating Small Value Digital Payments in Offline Mode: https://www.rbi.org.in/scripts/RTGS_Notification.aspx?Id=12215
- RBI - PSS Act FAQ: https://www.rbi.org.in/CommonPerson/english/scripts/FAQs.aspx?Id=420
- RBI - Payment Systems: https://www.rbi.org.in/scripts/paymentsystems.aspx
- NPCI - UPI Lite: https://www.npci.org.in/product/upi/upi-lite
- RBI Annual Report material on UPI Lite X / UPI 123PAY: https://rbi.org.in/scripts/PublicationsView.aspx?id=22459

## 7. Maintenance rule

Before every external pilot, partnership discussion, publication finalization, or production design review, re-check the current RBI/NPCI framework and update this document with the verification date and source links.

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
