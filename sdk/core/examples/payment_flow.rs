//! Runnable example demonstrating the full offline payment lifecycle in Rust using `resilientpay-core`.
//!
//! Run with:
//! ```bash
//! cargo run --example payment_flow
//! ```

use resilientpay_core::{
    crypto::{verify_envelope, Ed25519TestSigner, Ed25519Verifier, Signer},
    envelope::PaymentEnvelopeCore,
    ledger::{InMemoryLedger, LedgerEventInput, LocalLedger},
    money::Money,
    serialization::{encode_envelope_cbor, signing_input},
    types::{CredentialId, KeyId, MerchantId, TransactionId},
    PROTOCOL_VERSION,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ResilientPay Core SDK: End-to-End Payment Flow ===\n");

    // 1. Initialise Entities and Keys
    println!("[1] Setting up payer keypair and counterparty IDs...");
    let payer_seed = [0x77; 32];
    let signer = Ed25519TestSigner::from_seed(&payer_seed);
    let public_key = signer.public_key();
    let verifier = Ed25519Verifier::from_bytes(&public_key.to_bytes())?;

    let tx_id = TransactionId::generate();
    let credential_id = CredentialId::generate();
    let payer_key_id = KeyId::generate();
    let merchant_id = MerchantId::generate();

    let created_at = 1700000000;
    let expires_at = created_at + 3600;
    let nonce = [0x42; 16];
    let amount = Money::new(25000, "INR")?; // ₹250.00 (25,000 paise)
    let counter = 1u64;

    // 2. Payer constructs the canonical PaymentEnvelopeCore
    println!("[2] Payer constructs PaymentEnvelopeCore...");
    let envelope = PaymentEnvelopeCore::new(
        PROTOCOL_VERSION,
        tx_id,
        credential_id,
        payer_key_id,
        merchant_id,
        amount,
        counter,
        nonce,
        created_at,
        expires_at,
        None,
        None,
    )?;

    println!("    Transaction ID : {}", envelope.tx_id());
    println!("    Amount         : ₹{:.2}", envelope.amount().amount_minor() as f64 / 100.0);
    println!("    Sequence Counter: {}", envelope.counter());

    // 3. Payer signs the canonical domain-separated input
    println!("\n[3] Producing cryptographic Ed25519 signature over canonical CBOR...");
    let sign_input_bytes = signing_input(&envelope)?;
    println!("    Domain-separated signing input size: {} bytes", sign_input_bytes.len());
    let signature = signer.sign(&sign_input_bytes)?;
    println!("    Signature generated: {} bytes", signature.to_bytes().len());

    // 4. Serialize envelope to Canonical CBOR for transmission
    let cbor_payload = encode_envelope_cbor(&envelope)?;
    println!("    Canonical CBOR wire payload size   : {} bytes", cbor_payload.len());

    // 5. Merchant receives payload and verifies cryptographically
    println!("\n[4] Merchant receives payload over proximity transport (NFC/BLE/QR)...");
    println!("    Verifying Ed25519 signature against registered payer public key...");
    verify_envelope(&envelope, &signature, &verifier)?;
    println!("    ==> Cryptographic verification PASSED!");

    // 6. Record in Local Tamper-Evident Ledger
    println!("\n[5] Writing transaction to append-only local hash-chain ledger...");
    let mut ledger = InMemoryLedger::new();
    let entry = ledger.append(LedgerEventInput {
        tx_id: envelope.tx_id().clone(),
        event: resilientpay_core::state_machine::TransactionEvent::SignatureCreated,
        resulting_state: resilientpay_core::state_machine::TransactionState::Signed,
        recorded_at_unix_secs: created_at,
        payload_bytes: Some(cbor_payload),
    })?;

    println!("    Ledger entry #{} written with hash: {}", entry.sequence, hex::encode(entry.chain_hash));
    ledger.verify_chain()?;
    println!("    ==> Ledger hash-chain integrity verified 100% intact!");

    println!("\n=== Flow Completed Successfully! ===");
    Ok(())
}
