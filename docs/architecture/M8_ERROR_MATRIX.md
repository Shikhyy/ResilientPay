# M8 Error Matrix

Maps the strict domain errors through the platform stack to the user interface safely avoiding secret leakage.

| Domain Error (Rust) | Platform Error (Android) | Transport/Backend Result | User-Visible State |
| --- | --- | --- | --- |
| `ValidationError::OfflineBudgetExceeded` | `FfiException.ValidationFailure` | N/A (Prevented locally) | "Payment exceeds available offline balance." |
| `ValidationError::CounterReplay` | `FfiException.ValidationFailure` | Duplicate ignored by Backend | "This transaction was already processed." |
| `ValidationError::Expired` | `FfiException.ValidationFailure` | N/A (Rejected at Merchant) | "The transaction has expired. Please try again." |
| `CryptoError::SignatureInvalid` | `FfiException.SigningFailure` | Malformed/Rejected | "Security validation failed. Transaction rejected." |
| `CryptoError::KeyNotFound` | `FfiException.KeyUnavailable` | N/A | "Security credential missing. Please reconnect to sync." |
| `LedgerError::HashChainBroken` | `FfiException.LedgerFailure` | Re-sync forced on Backend | "Local history corrupted. Syncing with network..." |
| N/A (KeyStore Failure) | `ProviderException / KeystoreException` | N/A | "Device security module error. Restart application." |
| N/A (Transport timeout) | `IOException / BleTimeout` | Retry required | "Connection lost. Move devices closer and retry." |
| `StateError::InvalidTransition` | `FfiException.StateError` | N/A | "Internal error. Transaction cancelled." |

*Note: Raw stack traces, Keystore provider details, and private key identifiers must NEVER be piped into the User-Visible State or logged to logcat in plaintext.*
