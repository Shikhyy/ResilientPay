
# Merchant Screen State Matrix

| Screen | Loading | Normal | Offline | Pending | Error | Empty |
|---|---|---|---|---|---|---|
| Home | skeleton | receive action | local receive capability | sync count | data unavailable | no transactions |
| Request | preparing | QR/NFC ready | local methods | n/a | generation error | n/a |
| Received | verification | verified locally | stored locally | reconciliation pending | verification failure | n/a |
| History | skeleton | records | pending | pending sync | storage failure | no transactions |
| Conflict | loading evidence | n/a | n/a | conflict | conflict detail | n/a |
