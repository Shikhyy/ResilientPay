# M8 Vertical Slice Architecture

## 1. End-to-End Flow
The first operational vertical slice connects the full offline envelope lifecycle from the Payer's device to the Backend Reconciliation Engine.

**Flow:**
`Payer Android` -> `Android SDK` -> `Rust Core` -> `Android Keystore` -> `PaymentEnvelope` -> `Mock Transport` -> `Merchant Android` -> `Local Verification` -> `Local Ledger` -> `Go Backend` -> `PostgreSQL` -> `Reconciliation`

## 2. Component Contracts

### 2.1 Android SDK Contract
The public surface of the SDK provides an opaque client boundary avoiding the leakage of private keys or complex FFI error states into the UI.
- `initialize(config: SdkConfig)`: Provisions the hardware key if absent.
- `createPayment(merchantId: UUID, amount: Long): PaymentEnvelope`: Orchestrates the Rust core validation, counter increment, and keystore signature.
- `verify(envelope: PaymentEnvelope): VerificationResult`: Used by the Merchant to assert cryptographic validity, offline budget limit compliance, and expiration.

### 2.2 Mock Transport Contract
A replaceable, transport-independent interface strictly moving bytes.
- `send(payload: ByteArray): TransportResult`
- `receive(listener: (ByteArray) -> Unit)`
- `deliveryStatus(): Flow<DeliveryState>`
*Note: The domain layer remains strictly agnostic of whether the bytes flowed over Mock, NFC, or BLE.*

### 2.3 Merchant Verification Pipeline
When bytes are received, the Merchant app decodes and validates:
1. `decode(cborBytes)` -> `PaymentEnvelope`
2. `schema validation` (ensuring correct version and fields)
3. `signature verification` (Ed25519 public key verification)
4. `credential validation` (checking if credential ID is known/revoked locally)
5. `replay/counter validation` (strictly monotonic counter check)
6. `expiration validation` (timestamp check)
7. `policy/budget validation` (ensuring transaction is within outstanding limit)
8. `local acceptance/rejection` -> Outputs `TransactionState`
9. `ledger append` -> Appends to Local Hash-linked Ledger.

### 2.4 Backend Alignment
The Go Backend expects:
- `amount` as integer minor units.
- `ids` as standard UUID v4 strings.
- `counters` as unsigned 64-bit monotonically increasing integers.
- `timestamps` as Unix epoch seconds.

## 3. UI and Domain State Mapping
The Rust core operates the state machine. The Android UI reacts to these states:
- **Rust Domain:** `Created` -> **Android UI:** `Loading/Preparing`
- **Rust Domain:** `Signed` -> **Android UI:** `Signing/Transferring`
- **Rust Domain:** `Transferred` -> **Android UI:** `Offline Transfer Complete`
- **Rust Domain:** `Locally Verified` -> **Android UI:** `Accepted (Local)`
- **Rust Domain:** `Reconciled` -> **Android UI:** `Settled (Network)`
- **Rust Domain:** `Rejected` -> **Android UI:** `Failed / Rejected`

## 4. Idempotency and Retries
- Retries across the Mock Transport use the original signature; they do not increment the counter.
- The Backend uses the `TransactionId` (UUID) as the idempotency key. Duplicate submissions of the same envelope result in a `200 OK` (already reconciled).
