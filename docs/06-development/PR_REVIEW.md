
# Pull Request Review Standard

## Reviewer questions

### Correctness
Does the implementation match the task and underlying requirements?

### Architecture
Is the change located in the correct module? Is business logic duplicated?

### Protocol
Does the change alter serialization, state, authorization, credentials, counters, or reconciliation behavior?

### Security
Can an attacker exploit the new path? Are negative tests present?

### Reliability
What happens on retry, restart, timeout, duplication, and partial failure?

### Design
For UI changes:
- is the correct domain state shown?
- are loading and error states complete?
- is accessibility covered?
- does the change follow the design system?

### Evidence
Are claims supported by tests or measurements?

## Approval levels

Routine implementation changes may receive normal code review.

Protocol, security, architecture, and research-methodology changes require the appropriate specialist or human owner review.

## Reviewer output

Review comments should be:
- specific
- reproducible
- tied to a requirement or risk
- actionable

Avoid vague comments such as "make this cleaner" without identifying the expected behavior.
