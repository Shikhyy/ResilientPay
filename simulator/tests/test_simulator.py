"""
Simulator tests — domain model, crypto, transport, engine, scenarios.

Critical test: test_cross_sdk_cbor_vector
    Verifies the Python CBOR encoder produces bit-for-bit identical output
    to the Rust SDK (sdk/core/tests/protocol_test_vectors.rs) and Go backend
    (backend/internal/crypto/crypto_test.go) for the same frozen Gate 3 input.

    If this test fails, Python-generated envelopes cannot be verified by the
    Rust SDK or Go backend — the cross-language protocol is broken.
"""

from __future__ import annotations

import pytest

from resilientpay_sim.domain.crypto import (
    SIGNING_DOMAIN_SEPARATOR,
    encode_envelope_cbor,
    keypair_from_seed,
    sign_envelope,
    signing_input,
    verify_envelope,
)
from resilientpay_sim.domain.model import (
    Credential,
    CredentialState,
    Money,
    PaymentEnvelope,
    TransactionState,
)
from resilientpay_sim.engine import (
    PROTOCOL_VERSION,
    ReconciliationEngine,
    Simulator,
)
from resilientpay_sim.scenarios.catalogue import (
    duplicate_delivery,
    fully_offline,
    happy_path_internet,
    partial_loss_internet,
)
from resilientpay_sim.transport.channel import FaultProfile, TransportChannel, TransportKind

# ---------------------------------------------------------------------------
# Cross-SDK test vector (Gate 3 freeze)
#
# These hex values MUST match:
#   Rust:  sdk/core/tests/protocol_test_vectors.rs  EXPECTED_CBOR_HEX
#   Go:    backend/internal/crypto/crypto_test.go   crossSDKCBORHex
#
# Inputs (all deterministic):
#   TX UUID  : 00000000-0000-0000-0000-000000000001
#   CR UUID  : 00000000-0000-0000-0000-000000000002
#   Key UUID : 00000000-0000-0000-0000-000000000003
#   Mer UUID : 00000000-0000-0000-0000-000000000004
#   Nonce    : bytes([0x01] * 16)
#   Amount   : 150 paise
#   Currency : INR
#   Counter  : 1
#   Created  : 1_700_000_000
#   Expires  : 1_700_003_600
#   Protocol : 1
# ---------------------------------------------------------------------------

# 106 bytes — matches Rust SDK and Go backend Gate 3 frozen vector exactly
GATE3_CBOR_HEX_CLEAN = "8d015000000000000000000000000000000001500000000000000000000000000000000250000000000000000000000000000000035000000000000000000000000000000004189663494e520150010101010101010101010101010101011a6553f1001a6553ff10f6f6"

GATE3_SIGNING_SEED_HEX = "42" * 32  # [0x42; 32]
GATE3_PUBLIC_KEY_HEX = "2152f8d19b791d24453242e15f2eab6cb7cffa7b6a5ed30097960e069881db12"
GATE3_SIGNATURE_HEX = (
    "04bbdb205dd371101fb2ad439d5f1d38cfc9146f7edd157a22f3476c42b0112359"
    "c46f1f0ea16aa1300cff1576628264bcb2f33b1d4ac2d6d8f592e16406ee07"
)


def _vector_envelope() -> PaymentEnvelope:
    return PaymentEnvelope(
        protocol_version=1,
        tx_id="00000000-0000-0000-0000-000000000001",
        credential_id="00000000-0000-0000-0000-000000000002",
        payer_key_id="00000000-0000-0000-0000-000000000003",
        merchant_id="00000000-0000-0000-0000-000000000004",
        amount=Money(amount_minor=150, currency="INR"),
        counter=1,
        nonce=bytes([0x01] * 16),
        created_at_unix=1_700_000_000,
        expires_at_unix=1_700_003_600,
        previous_event_hash=None,
        risk_class=None,
    )


class TestCrossSDKCBORVector:
    """Byte-level cross-language CBOR encoding verification."""

    def test_cbor_matches_frozen_gate3_vector(self) -> None:
        """Python encoder must produce bit-for-bit identical CBOR to Rust/Go."""
        env = _vector_envelope()
        actual = encode_envelope_cbor(env)
        expected = bytes.fromhex(GATE3_CBOR_HEX_CLEAN)

        assert len(actual) == len(expected), (
            f"CBOR length mismatch: expected {len(expected)}, got {len(actual)}\n"
            f"Expected: {expected.hex()}\n"
            f"Actual:   {actual.hex()}"
        )
        assert actual == expected, (
            "CBOR bytes differ from frozen Gate 3 vector — Python encoder has diverged from Rust/Go.\n"
            f"Expected: {expected.hex()}\n"
            f"Actual:   {actual.hex()}"
        )

    def test_cbor_length_is_106_bytes(self) -> None:
        env = _vector_envelope()
        cbor = encode_envelope_cbor(env)
        assert len(cbor) == 106, f"Expected 106 bytes, got {len(cbor)}"

    def test_signing_input_starts_with_domain_separator(self) -> None:
        env = _vector_envelope()
        si = signing_input(env)
        assert si.startswith(SIGNING_DOMAIN_SEPARATOR), (
            f"signing input must start with {SIGNING_DOMAIN_SEPARATOR!r}"
        )

    def test_signing_input_length_is_139_bytes(self) -> None:
        env = _vector_envelope()
        si = signing_input(env)
        assert len(si) == 139, f"Expected 139 bytes (33 domain sep + 106 CBOR), got {len(si)}"

    def test_public_key_matches_frozen_vector(self) -> None:
        seed = bytes.fromhex(GATE3_SIGNING_SEED_HEX)
        _, pub = keypair_from_seed(seed)
        assert pub.hex() == GATE3_PUBLIC_KEY_HEX, (
            f"Public key mismatch:\nExpected: {GATE3_PUBLIC_KEY_HEX}\nActual:   {pub.hex()}"
        )

    def test_signature_matches_frozen_vector(self) -> None:
        seed = bytes.fromhex(GATE3_SIGNING_SEED_HEX)
        _, pub = keypair_from_seed(seed)
        env = _vector_envelope()
        sig = sign_envelope(env, seed)
        assert sig.hex() == GATE3_SIGNATURE_HEX, (
            f"Signature mismatch:\nExpected: {GATE3_SIGNATURE_HEX}\nActual:   {sig.hex()}"
        )

    def test_signature_verifies_with_corresponding_public_key(self) -> None:
        seed = bytes.fromhex(GATE3_SIGNING_SEED_HEX)
        _, pub = keypair_from_seed(seed)
        env = _vector_envelope()
        sig = sign_envelope(env, seed)
        # Should not raise
        verify_envelope(env, sig, pub)


# ---------------------------------------------------------------------------
# Domain model tests
# ---------------------------------------------------------------------------


class TestMoney:
    def test_valid_amount(self) -> None:
        m = Money(amount_minor=100, currency="INR")
        assert m.amount_minor == 100

    def test_zero_valid(self) -> None:
        m = Money(amount_minor=0, currency="INR")
        assert m.amount_minor == 0

    def test_negative_raises(self) -> None:
        with pytest.raises(ValueError, match="non-negative"):
            Money(amount_minor=-1, currency="INR")

    def test_float_raises(self) -> None:
        with pytest.raises(TypeError):
            Money(amount_minor=1.5, currency="INR")  # type: ignore[arg-type]

    def test_empty_currency_raises(self) -> None:
        with pytest.raises(ValueError):
            Money(amount_minor=100, currency="")

    def test_long_currency_raises(self) -> None:
        with pytest.raises(ValueError):
            Money(amount_minor=100, currency="TOOLONGCODE")


class TestTransactionState:
    def test_terminal_states(self) -> None:
        assert TransactionState.RECONCILED.is_terminal
        assert TransactionState.REJECTED.is_terminal
        assert TransactionState.CONFLICT.is_terminal

    def test_non_terminal_states(self) -> None:
        for s in (
            TransactionState.CREATED,
            TransactionState.VALIDATING,
            TransactionState.AUTHORIZED,
            TransactionState.SIGNED,
            TransactionState.TRANSFERRED,
            TransactionState.RECEIVED,
            TransactionState.LOCALLY_VERIFIED,
            TransactionState.LOCALLY_RECORDED,
            TransactionState.SYNC_PENDING,
        ):
            assert not s.is_terminal, f"{s} should not be terminal"


# ---------------------------------------------------------------------------
# Crypto tests
# ---------------------------------------------------------------------------


class TestSignAndVerify:
    def test_round_trip(self) -> None:
        env = _vector_envelope()
        seed, pub = keypair_from_seed(bytes(range(32)))
        sig = sign_envelope(env, seed)
        verify_envelope(env, sig, pub)  # must not raise

    def test_tampered_amount_rejected(self) -> None:
        from cryptography.exceptions import InvalidSignature

        env = _vector_envelope()
        seed, pub = keypair_from_seed(bytes(range(32)))
        sig = sign_envelope(env, seed)

        import attr

        tampered = attr.evolve(env, amount=Money(amount_minor=1, currency="INR"))
        with pytest.raises(InvalidSignature):
            verify_envelope(tampered, sig, pub)

    def test_deterministic_signature(self) -> None:
        env = _vector_envelope()
        seed, _ = keypair_from_seed(bytes([0x99] * 32))
        sig1 = sign_envelope(env, seed)
        sig2 = sign_envelope(env, seed)
        assert sig1 == sig2, "Ed25519 must be deterministic"


# ---------------------------------------------------------------------------
# Transport channel tests
# ---------------------------------------------------------------------------


class TestTransportChannel:
    def test_no_fault_delivers(self) -> None:
        env = _vector_envelope()
        ch = TransportChannel(kind=TransportKind.INTERNET)
        delivered = ch.send(env, sim_time=1_700_000_000)
        assert delivered
        assert len(ch.delivered) == 1

    def test_100_pct_loss(self) -> None:
        import random

        env = _vector_envelope()
        ch = TransportChannel(
            kind=TransportKind.INTERNET,
            fault_profile=FaultProfile(loss_probability=1.0),
            rng=random.Random(42),
        )
        delivered = ch.send(env, sim_time=1_700_000_000)
        assert not delivered
        assert len(ch.delivered) == 0

    def test_100_pct_duplicate(self) -> None:
        import random

        env = _vector_envelope()
        ch = TransportChannel(
            kind=TransportKind.NFC,
            fault_profile=FaultProfile(duplicate_probability=1.0),
            rng=random.Random(42),
        )
        ch.send(env, sim_time=1_700_000_000)
        assert len(ch.delivered) == 2


# ---------------------------------------------------------------------------
# Reconciliation engine tests
# ---------------------------------------------------------------------------


class TestReconciliationEngine:
    def _make_cred_and_payer(self) -> tuple[Credential, bytes]:
        seed = bytes([0x11] * 32)
        _, pub = keypair_from_seed(seed)
        cred = Credential(
            public_key_bytes=pub,
            issued_at_unix=1_700_000_000,
            expires_at_unix=1_700_003_600,
            state=CredentialState.ACTIVE,
        )
        return cred, seed

    def _make_envelope(self, cred: Credential, seed: bytes, counter: int = 1) -> PaymentEnvelope:
        import attr

        env = PaymentEnvelope(
            protocol_version=PROTOCOL_VERSION,
            credential_id=cred.credential_id,
            payer_key_id=cred.subject_key_id,
            merchant_id="00000000-0000-0000-0000-000000000099",
            amount=Money(amount_minor=100, currency="INR"),
            counter=counter,
            nonce=bytes([0x01] * 16),
            created_at_unix=1_700_000_000,
            expires_at_unix=1_700_003_600,
        )
        sig = sign_envelope(env, seed)
        return attr.evolve(env, signature_bytes=sig)

    def test_happy_path_accepted(self) -> None:
        engine = ReconciliationEngine()
        cred, seed = self._make_cred_and_payer()
        engine.register_credential(cred)
        env = self._make_envelope(cred, seed)
        state, reason = engine.reconcile(env, now_unix=1_700_001_000)
        assert state == TransactionState.RECONCILED

    def test_duplicate_is_already_known(self) -> None:
        engine = ReconciliationEngine()
        cred, seed = self._make_cred_and_payer()
        engine.register_credential(cred)
        env = self._make_envelope(cred, seed)
        engine.reconcile(env, now_unix=1_700_001_000)
        state, reason = engine.reconcile(env, now_unix=1_700_001_000)
        assert state == TransactionState.RECONCILED
        assert "already_known" in reason

    def test_expired_credential_rejected(self) -> None:
        engine = ReconciliationEngine()
        cred, seed = self._make_cred_and_payer()
        engine.register_credential(cred)
        env = self._make_envelope(cred, seed)
        state, _ = engine.reconcile(env, now_unix=1_700_099_999)  # past expiry
        assert state == TransactionState.REJECTED

    def test_unknown_credential_rejected(self) -> None:
        engine = ReconciliationEngine()
        cred, seed = self._make_cred_and_payer()
        # Do NOT register the credential
        env = self._make_envelope(cred, seed)
        state, _ = engine.reconcile(env, now_unix=1_700_001_000)
        assert state == TransactionState.REJECTED

    def test_duplicate_counter_different_tx_id_is_conflict(self) -> None:
        import attr

        engine = ReconciliationEngine()
        cred, seed = self._make_cred_and_payer()
        engine.register_credential(cred)

        # First transaction with counter=1
        env1 = self._make_envelope(cred, seed, counter=1)
        state1, reason1 = engine.reconcile(env1, now_unix=1_700_001_000)
        assert state1 == TransactionState.RECONCILED
        assert reason1 == "accepted"

        # Second transaction with same credential, same counter=1, but different tx_id
        env2 = attr.evolve(
            env1,
            tx_id="00000000-0000-0000-0000-000000000002",
            nonce=bytes([0x02] * 16),
            signature_bytes=None,
        )
        sig2 = sign_envelope(env2, seed)
        env2 = attr.evolve(env2, signature_bytes=sig2)

        state2, reason2 = engine.reconcile(env2, now_unix=1_700_001_000)
        assert state2 == TransactionState.CONFLICT
        assert "duplicate counter" in reason2


# ---------------------------------------------------------------------------
# Scenario / end-to-end tests
# ---------------------------------------------------------------------------


class TestScenarios:
    def test_happy_path_all_reconciled(self) -> None:
        cfg = happy_path_internet(seed=42)
        result = Simulator(cfg).run()
        assert result.total_transactions == cfg.num_transactions
        assert result.total_reconciled == cfg.num_transactions
        assert result.total_rejected == 0
        assert result.total_conflicts == 0
        assert result.total_transport_losses == 0

    def test_partial_loss_reduces_reconciled(self) -> None:
        cfg = partial_loss_internet(seed=42)
        result = Simulator(cfg).run()
        assert result.total_transactions == cfg.num_transactions
        # With 30% loss, we expect some to be lost
        assert result.total_transport_losses > 0
        assert result.total_reconciled < cfg.num_transactions

    def test_fully_offline_zero_reconciled(self) -> None:
        cfg = fully_offline(seed=42)
        result = Simulator(cfg).run()
        assert result.total_transport_losses == cfg.num_transactions
        assert result.total_reconciled == 0

    def test_duplicate_delivery_no_conflicts(self) -> None:
        cfg = duplicate_delivery(seed=42)
        result = Simulator(cfg).run()
        # Duplicates should be ALREADY_KNOWN (not conflicts)
        assert result.total_conflicts == 0
        assert result.total_reconciled == cfg.num_transactions

    def test_determinism_same_seed_same_result(self) -> None:
        cfg1 = happy_path_internet(seed=99)
        cfg2 = happy_path_internet(seed=99)
        r1 = Simulator(cfg1).run()
        r2 = Simulator(cfg2).run()
        assert r1.total_reconciled == r2.total_reconciled
        assert r1.total_transport_losses == r2.total_transport_losses
        assert len(r1.event_log) == len(r2.event_log)

    def test_different_seeds_may_differ(self) -> None:
        # With partial loss, different seeds give different loss counts (probabilistic)
        cfg_a = partial_loss_internet(seed=1)
        cfg_b = partial_loss_internet(seed=9999)
        ra = Simulator(cfg_a).run()
        rb = Simulator(cfg_b).run()
        # Not guaranteed, but almost certainly true with different seeds
        # Just verify both are valid (no crash)
        assert ra.total_transactions == rb.total_transactions

    def test_result_includes_reproducibility_fields(self) -> None:
        cfg = happy_path_internet(seed=42)
        result = Simulator(cfg).run()
        assert result.scenario_id == cfg.scenario_id
        assert result.seed == cfg.seed
        assert result.protocol_version == PROTOCOL_VERSION
        assert result.start_time_unix == cfg.start_time_unix
        assert len(result.event_log) > 0
