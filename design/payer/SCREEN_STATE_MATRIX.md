
# Payer Screen State Matrix

| Screen | Loading | Normal | Offline | Pending | Error | Empty |
|---|---|---|---|---|---|---|
| Home | skeleton | capability visible | offline capability | sync count | data unavailable | no activity |
| Amount | form skeleton | editable | bounded offline notice | n/a | validation | n/a |
| Payment | preparing | authorization | local authorization | sync pending | verification failure | n/a |
| Transaction | skeleton | reconciled | stored locally | sync pending | conflict/rejection | n/a |
| History | skeleton | records | pending records | visible | storage failure | no transactions |

## Rule

The matrix is part of the implementation contract. A screen is incomplete if a supported state has no defined presentation.
