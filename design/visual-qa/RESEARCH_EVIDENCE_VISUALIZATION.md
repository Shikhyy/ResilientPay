<!--
ResilientPay professional documentation package.
Source document retained and reorganized from: design-source/RESEARCH_EVIDENCE_VISUALIZATION.md
Authority: Design source of truth
This file is normative unless explicitly marked as informative in its body.
-->
# Research Evidence Visualization

> **Document role:** Normative UX/visual specification.

## Purpose

Research visuals should communicate measured evidence, not decorative analytics.

## Chart types

Preferred:
- line charts for latency over time
- bar charts for comparative transport measurements
- scatter plots for risk score behavior
- tables for exact protocol results
- state timelines for failure recovery
- Sankey-like or flow diagrams only when the quantity represented justifies them

## Chart principles

- label units explicitly
- show sample size
- show test conditions
- show confidence intervals where statistically appropriate
- state limitations
- do not fabricate smooth trends
- do not hide failed runs
- preserve zero baselines when appropriate

## Example

Transport comparison:

Metric: reconciliation latency in seconds

Conditions:
- 100 simulated transactions
- 10% message duplication
- 5% message loss
- fixed backend environment

Report:
median
p95
failure rate
retries
final reconciliation rate

Research charts must be linked to an experiment identifier.
