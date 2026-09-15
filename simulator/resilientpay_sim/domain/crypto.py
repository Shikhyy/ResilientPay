"""
Cryptographic operations for the simulator.

Uses the `cryptography` library (PyCA) for Ed25519 — the same algorithm as
ed25519-dalek (Rust) and Go's crypto/ed25519. All three use the same RFC 8032
Ed25519 specification, so a key pair generated here can be verified by the
Rust SDK or Go backend.

CBOR encoding mirrors serialization.rs and backend/internal/crypto/crypto.go
exactly. The field order and type encoding are normative — see the Gate 3
frozen test vectors in sdk/core/tests/protocol_test_vectors.rs.

Security rules:
    - Never log or print private key bytes.
    - Private keys exist only in PayerDevice.private_key_bytes (32-byte seed).
    - Signing domain separator MUST match Rust SDK and Go backend.
"""

from __future__ import annotations

import uuid as uuid_mod

import cbor2
from cryptography.hazmat.primitives.asymmetric.ed25519 import (
    Ed25519PrivateKey,
    Ed25519PublicKey,
)
from cryptography.hazmat.primitives.serialization import (
    Encoding,
    NoEncryption,
    PrivateFormat,
    PublicFormat,
)

from resilientpay_sim.domain.model import PaymentEnvelope

# ---------------------------------------------------------------------------
# Domain separator — MUST match serialization.rs and Go crypto.go
# ---------------------------------------------------------------------------

SIGNING_DOMAIN_SEPARATOR: bytes = b"resilientpay:payment-envelope:v1:"

# ---------------------------------------------------------------------------
# CBOR canonical encoder
# ---------------------------------------------------------------------------


def _uuid_to_bytes(uuid_str: str) -> bytes:
    """Return the 16 raw bytes of a UUID (not the hyphenated text form).

    This matches the Rust SDK:
        Value::Bytes(envelope.tx_id().as_uuid().as_bytes().to_vec())
    """
    return uuid_mod.UUID(uuid_str).bytes


def encode_envelope_cbor(env: PaymentEnvelope) -> bytes:
    """Encode a PaymentEnvelope as the canonical 13-element CBOR array.

    Field order is fixed per PAYMENT_PROTOCOL.md §2 and matches:
        serialization.rs encode_envelope_cbor()
        backend/internal/crypto/crypto.go encodeToCBOR()

    The CBOR encoding is NOT tagged and uses definite-length encoding.
    cbor2 uses shortest-form integers by default, matching ciborium and fxamacker.
    """
    previous_event_hash = env.previous_event_hash  # 32 bytes or None → CBOR null
    risk_class = env.risk_class  # str or None → CBOR null

    array = [
        env.protocol_version,  # [0]  uint
        _uuid_to_bytes(env.tx_id),  # [1]  bstr 16
        _uuid_to_bytes(env.credential_id),  # [2]  bstr 16
        _uuid_to_bytes(env.payer_key_id),  # [3]  bstr 16
        _uuid_to_bytes(env.merchant_id),  # [4]  bstr 16
        env.amount.amount_minor,  # [5]  uint
        env.amount.currency,  # [6]  tstr
        env.counter,  # [7]  uint
        env.nonce,  # [8]  bstr 16
        env.created_at_unix,  # [9]  int
        env.expires_at_unix,  # [10] int
        previous_event_hash,  # [11] bstr 32 or null
        risk_class,  # [12] tstr or null
    ]

    return cbor2.dumps(array, timezone=None, canonical=False)


def signing_input(env: PaymentEnvelope) -> bytes:
    """Return: SIGNING_DOMAIN_SEPARATOR || cbor_bytes.

    This is the byte sequence that must be signed by the payer's Ed25519 key.
    """
    return SIGNING_DOMAIN_SEPARATOR + encode_envelope_cbor(env)


# ---------------------------------------------------------------------------
# Key generation
# ---------------------------------------------------------------------------


def generate_keypair() -> tuple[bytes, bytes]:
    """Generate a fresh Ed25519 key pair.

    Returns:
        (private_seed_bytes, public_key_bytes) — both 32 bytes.

    The private_seed_bytes is the 32-byte RFC 8032 seed (first 32 bytes of the
    64-byte expanded private key). This matches the seed used by:
        Rust: Ed25519TestSigner::from_seed(&[u8; 32])
        Go:   ed25519.NewKeyFromSeed(seed)
    """
    priv = Ed25519PrivateKey.generate()
    # Extract 32-byte seed
    private_bytes = priv.private_bytes(Encoding.Raw, PrivateFormat.Raw, NoEncryption())
    public_bytes = priv.public_key().public_bytes(Encoding.Raw, PublicFormat.Raw)
    return private_bytes, public_bytes


def keypair_from_seed(seed: bytes) -> tuple[bytes, bytes]:
    """Derive an Ed25519 key pair from a 32-byte seed deterministically.

    Equivalent to Rust Ed25519TestSigner::from_seed(&seed).
    """
    if len(seed) != 32:
        raise ValueError(f"seed must be 32 bytes, got {len(seed)}")
    priv = Ed25519PrivateKey.from_private_bytes(seed)
    public_bytes = priv.public_key().public_bytes(Encoding.Raw, PublicFormat.Raw)
    return seed, public_bytes


# ---------------------------------------------------------------------------
# Signing and verification
# ---------------------------------------------------------------------------


def sign_envelope(env: PaymentEnvelope, private_seed: bytes) -> bytes:
    """Sign a PaymentEnvelope with the given Ed25519 private seed.

    Returns the 64-byte Ed25519 signature over the canonical signing input.
    """
    msg = signing_input(env)
    priv = Ed25519PrivateKey.from_private_bytes(private_seed)
    return priv.sign(msg)


def verify_envelope(
    env: PaymentEnvelope,
    signature_bytes: bytes,
    public_key_bytes: bytes,
) -> None:
    """Verify the Ed25519 signature over the canonical signing input.

    Raises:
        cryptography.exceptions.InvalidSignature — if the signature is invalid.
            This is a security rejection; callers MUST NOT silently ignore it.
    """
    msg = signing_input(env)
    pub = Ed25519PublicKey.from_public_bytes(public_key_bytes)
    pub.verify(signature_bytes, msg)  # raises InvalidSignature on failure
