# ResilientPay Core SDK (`resilientpay-core`)

[![Crates.io](https://img.shields.io/badge/crates.io-v0.1.0-orange.svg)](https://crates.io)
[![Documentation](https://docs.rs/resilientpay-core/badge.svg)](https://docs.rs/resilientpay-core)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

`resilientpay-core` is the normative, transport-independent security and protocol SDK for **ResilientPay** — a connectivity-resilient retail payment system designed for low-bandwidth, high-latency, and zero-connectivity retail environments.

This crate is intentionally pure Rust: it contains **zero Android, network, or filesystem dependencies**. It exposes clean APIs for building, signing, validating, and local-ledgering store-and-forward payment envelopes, as well as multi-language foreign function interface (FFI) bindings via Mozilla UniFFI.

---

## Architecture & Principles

1. **Transport Independence**: The core SDK deals only with canonical bytes, cryptographic signatures, and state machine transitions. Network, proximity radio (NFC, BLE, QR), and telecommunication transports (SMS) are external adapters.
2. **Deterministic Validation**: Every payment envelope undergoes ordered validation checks (protocol version, counter monotonicity, bounded budget, expiry, merchant binding) before cryptographic authorization.
3. **Canonical Serialization (RFC 8949 CBOR)**: Envelopes encode into a deterministic, strictly ordered 106-byte CBOR payload (`RESILIENTPAY-PAYMENT-V1:` domain separated) to ensure signature stability across architectures.
4. **Hardware Key Isolation**: Cryptographic seeds and private keys remain sealed inside platform secure enclaves (e.g., Android Keystore, iOS Secure Enclave, or HSMs). The SDK calls an abstract `KeyManager` callback to sign payloads without ever handling raw private keys.
5. **Tamper-Evident Local Ledger**: An append-only SHA-256 hash-chained state ledger provides instant $O(1)$ tamper detection for offline transactions prior to cloud reconciliation.

---

## Installation

Add `resilientpay-core` to your `Cargo.toml`:

```toml
[dependencies]
resilientpay-core = { version = "0.1.0" }
```

---

## Quickstart Guide

### 1. Payer Flow: Authorizing and Signing an Offline Payment

```rust
use resilientpay_core::{
    crypto::{Ed25519TestSigner, Signer},
    envelope::{EnvelopeBuilder, PaymentEnvelopeCore},
    money::Money,
    serialization::{encode_envelope_cbor, signing_input},
    types::{CredentialId, KeyId, MerchantId, TransactionId},
};
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
    
    // 1. Build the transaction envelope
    let envelope = EnvelopeBuilder::new()
        .tx_id(TransactionId::new_random())
        .credential_id(CredentialId::new_random())
        .payer_key_id(KeyId::new_random())
        .merchant_id(MerchantId::new_random())
        .amount(Money::new(15000, "INR")?) // ₹150.00 (15,000 paise)
        .counter(1)                        // Monotonically increasing sequence
        .nonce([0x42; 16])                 // 16-byte cryptographically secure nonce
        .created_at_unix(now)
        .expires_at_unix(now + 3600)       // 1 hour offline validity
        .build()?;

    // 2. Compute canonical signing input (Domain separator + Canonical CBOR)
    let sign_input = signing_input(&envelope)?;

    // 3. Produce signature via KeyManager / Signer
    let signer = Ed25519TestSigner::from_seed(&[0x11; 32]);
    let signature = signer.sign(&sign_input)?;

    // 4. Serialize envelope to compact Canonical CBOR for over-the-air transfer
    let cbor_bytes = encode_envelope_cbor(&envelope)?;
    println!("Encoded envelope size: {} bytes", cbor_bytes.len());

    Ok(())
}
```

### 2. Merchant Flow: Offline Cryptographic Verification

```rust
use resilientpay_core::{
    crypto::{verify_envelope, Ed25519Verifier, Signature},
    envelope::PaymentEnvelopeCore,
    serialization::decode_envelope_cbor_fields,
};

fn verify_received_payment(
    cbor_bytes: &[u8],
    signature_bytes: &[u8],
    payer_public_key_bytes: &[u8; 32],
) -> Result<bool, Box<dyn std::error::Error>> {
    // 1. Decode canonical CBOR envelope
    let decoded = decode_envelope_cbor_fields(cbor_bytes)?;
    
    // 2. Reconstruct domain envelope
    let envelope = PaymentEnvelopeCore::from_decoded_fields(decoded)?;

    // 3. Verify RFC 8032 Ed25519 signature over domain-separated input
    let verifier = Ed25519Verifier::from_bytes(payer_public_key_bytes)?;
    let signature = Signature::from_bytes(signature_bytes)?;
    
    verify_envelope(&envelope, &signature, &verifier)?;
    Ok(true)
}
```

### 3. Local Tamper-Evident Ledger

```rust
use resilientpay_core::ledger::{InMemoryLedger, LedgerEventInput, LocalLedger};

fn record_audit_trail() -> Result<(), Box<dyn std::error::Error>> {
    let mut ledger = InMemoryLedger::new();

    // Append payment event
    let event = ledger.append(LedgerEventInput {
        event_type: "PAYMENT_AUTHORIZED_LOCALLY".to_string(),
        tx_id: Some("f47ac10b-58cc-4372-a567-0e02b2c3d479".to_string()),
        payload: b"signed_cbor_envelope_bytes".to_vec(),
    })?;

    // Verify cryptographic hash chain integrity across all records
    assert!(ledger.verify_chain()?);
    println!("Ledger verified! Latest chain hash: {}", hex::encode(ledger.last_chain_hash()));

    Ok(())
}
```

---

## UniFFI Cross-Language Bindings

`resilientpay-core` ships with out-of-the-box UniFFI bindings:

```bash
# Generate Kotlin bindings for Android
cargo run --bin uniffi-bindgen generate \
    --library target/release/libresilientpay_core.dylib \
    --language kotlin \
    --out-dir ../android/bindings

# Generate Swift bindings for iOS
cargo run --bin uniffi-bindgen generate \
    --library target/release/libresilientpay_core.dylib \
    --language swift \
    --out-dir ../ios/bindings
```

---

## Security Invariants

When integrating `resilientpay-core` into client applications:
1. **Never use floating point for currency**: Always use `Money::new(minor_units, "INR")` where units are integer paise.
2. **Enforce Monotonic Counters**: Never reuse a counter. Duplicate or regressive counters are rejected as double-spend attempts.
3. **Never transmit raw private keys**: Keys must be generated and stored inside hardware-backed keystores.
4. **Always verify domain separation**: All signing inputs must use `RESILIENTPAY-PAYMENT-V1:`.

---

## License

Dual-licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
