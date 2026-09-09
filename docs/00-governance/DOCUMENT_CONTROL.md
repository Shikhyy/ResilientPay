
# Document Control

## Purpose

This repository uses documentation as an engineering control surface. The goal is to prevent a coding agent, developer, or future contributor from treating implementation convenience as an undocumented product decision.

## Document classes

### Normative

Defines behavior that implementation must follow.

Examples:
- requirements
- threat model
- crypto specification
- payment protocol
- state machine
- API contract

### Informative

Explains context without directly defining implementation behavior.

Examples:
- research narrative
- background explanation
- conceptual diagrams

### Procedural

Defines how engineering work is performed.

Examples:
- agent workflow
- release process
- testing process
- change control

### Evidence

Records measurements, experiment outputs, or review results.

## Change rule

A change to a normative document must answer:
1. What behavior changes?
2. Why?
3. What requirements are affected?
4. What security assumptions change?
5. What tests need to change?
6. Is backward compatibility affected?
7. Does an ADR need to be added or updated?

## Status

Every important document should indicate whether it is:
- draft
- under review
- approved
- superseded
- deprecated

An implementation should not treat a draft as final protocol authority unless the task explicitly says it is an experiment.
