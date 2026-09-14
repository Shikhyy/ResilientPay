# ADR 012: Android Rust Crypto FFI Boundary

## Status
Proposed

## Context
Milestone M7 requires integrating the Rust `resilientpay-core` SDK into the Kotlin Android application. The critical constraint is that the Ed25519 private key must reside within the Android Keystore (Trusted Execution Environment / Secure Enclave) and its raw bytes must NEVER be exported into Kotlin application memory, JNI arguments, or Rust memory. 

We must define a secure FFI (Foreign Function Interface) boundary that allows the Rust state machine to orchestrate the transaction and canonicalize the payload (CBOR), while delegating the actual cryptographic signature operation back to the Android Keystore.

## 1. Security Boundary & Key Ownership
- **Physical Key Location**: Android Keystore (TEE/SE).
- **Key Ownership**: The Android OS/Keystore owns the private key material.
- **Signing Requester**: The Rust SDK generates the canonical CBOR payload (with domain separators) and requests the signature.
- **Data crossing JNI**: 
  - To Kotlin: `key_id` (String UUID), `payload` (byte array).
  - To Rust: `signature` (byte array), `public_key` (byte array).
- **Data explicitly forbidden across JNI**: Private key bytes.

## 2. FFI Model Selection
We evaluated raw JNI versus Mozilla UniFFI.
- **Decision**: **Mozilla UniFFI**.
- **Rationale**: 
  - UniFFI automates the JNI boilerplate, explicitly preventing memory leaks related to JNI local/global references.
  - It supports "Callback Interfaces", allowing us to define a Rust `KeyManager` trait that is implemented natively in Kotlin.
  - It automatically handles thread-safety, ABI stability, and Rust `Result` to Kotlin `Exception` error mappings.
  - Future iOS (Swift) integration will be freely provided by the same UniFFI definitions.

## 3. FFI Contract Definition
The contract will be defined using UniFFI's UDL (or attribute macros). Conceptually:

```rust
// Rust defining the callback interface expected from Kotlin
#[uniffi::export(callback_interface)]
pub trait AndroidKeyManager: Send + Sync {
    fn get_public_key(&self, key_id: String) -> Result<Vec<u8>, CryptoError>;
    fn sign(&self, key_id: String, payload: Vec<u8>) -> Result<Vec<u8>, CryptoError>;
}

// Rust exporting the entry point
#[uniffi::export]
pub fn create_transaction(
    key_manager: Box<dyn AndroidKeyManager>,
    payer_key_id: String,
    merchant_id: String,
    amount_minor: u64
) -> Result<Vec<u8>, DomainError> {
    // ... constructs payload, calls key_manager.sign(), returns envelope bytes
}
```

- **Memory Ownership**: Rust owns the state machine. Kotlin owns the `KeyManager` instance and passes it by reference/box across the boundary. UniFFI manages the proxy lifecycle.
- **Thread-Safety**: The callback must be `Send + Sync`. Android's Keystore operations are thread-safe.

## 4. Signing Direction
**Rust -> Kotlin Callback**.
1. Kotlin calls `create_transaction` passing a `KeyManager` proxy.
2. Rust prepares canonical signing bytes (CBOR).
3. Rust invokes `KeyManager::sign(payload)`.
4. Kotlin receives the callback, fetches the `java.security.Signature` object from Android Keystore using `key_id`, and signs the payload.
5. Kotlin returns the signature bytes to Rust.
6. Rust verifies the signature locally (to ensure hardware correctness) and wraps it into the final `PaymentEnvelope`.

## 5. Android Toolchain Requirements & Blocker Mitigation
Currently, the build system lacks Java/Gradle.
- **Category A (No Android Toolchain)**: Writing the Rust UniFFI definitions, compiling the Rust dynamic library (`.so`), generating the Kotlin bindings (`uniffi-bindgen`), and running Rust-side mock tests.
- **Category B (Requires Android Toolchain)**: Compiling the generated Kotlin against the Android SDK, executing `assembleDebug`.
- **Category C (Requires Device/Emulator)**: Keystore operations require an emulator or physical device. Roboelectric can mock some aspects, but actual TEE integration tests require `connectedAndroidTest`.

## 6. Test Strategy
Before full implementation, tests must verify:
- **Rust Initialization**: `init` does not panic.
- **Key Not Found**: Kotlin throwing a `KeyNotFoundException` translates to a Rust `CryptoError`.
- **Memory Leaks**: Rapidly calling the signing loop in a stress test does not exhaust JNI local references.
- **Security Constraint**: Test that the payload passed to the callback exactly matches the canonical CBOR without containing any private material.
