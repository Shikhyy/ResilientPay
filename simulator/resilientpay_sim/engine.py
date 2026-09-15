"""
Simulator engine — scenario runner and in-process reconciliation.

The engine drives a single simulation run to completion. It is:
    - Deterministic when seeded (every stochastic choice uses self.rng)
    - Time-based: all events record a sim_time (integer unix seconds)
    - Auditable: all events appended to self.event_log
    - Reproducible: the run_id, seed, config, and protocol_version are
      recorded in the SimulationResult

Reconciliation in the simulator is performed by an in-process
ReconciliationEngine that mirrors the normative 8-step ingestion flow
(RECONCILIATION_SPEC.md §3) without network I/O. This allows offline
scenario testing without a running backend server.

Protocol reference:
    docs/07-research-experiments/SIMULATION.md
    docs/05-protocol/RECONCILIATION_SPEC.md §3
"""

from __future__ import annotations

import random
import uuid
from dataclasses import dataclass, field

import attr
from cryptography.exceptions import InvalidSignature

from resilientpay_sim.domain.crypto import (
    generate_keypair,
    sign_envelope,
    verify_envelope,
)
from resilientpay_sim.domain.model import (
    ConnectivityState,
    Credential,
    CredentialState,
    Merchant,
    Money,
    PayerDevice,
    PaymentEnvelope,
    SimEvent,
    SimEventKind,
    TransactionState,
)
from resilientpay_sim.risk.model import RuleBasedRiskModel
from resilientpay_sim.transport.channel import FaultProfile, TransportChannel, TransportKind

# ---------------------------------------------------------------------------
# Simulation configuration
# ---------------------------------------------------------------------------

PROTOCOL_VERSION: int = 1


@dataclass
class ScenarioConfig:
    """Fully serialisable configuration for a single simulation scenario.

    A scenario is reproducible from (scenario_id, seed, config) alone.
    """

    scenario_id: str
    seed: int
    num_payers: int = 10
    num_merchants: int = 3
    num_transactions: int = 20
    connectivity: ConnectivityState = ConnectivityState.C3
    transport_kind: TransportKind = TransportKind.INTERNET
    fault_profile: FaultProfile = field(default_factory=FaultProfile)
    # Credential policy defaults
    credential_valid_seconds: int = 3_600
    max_value_per_tx_minor: int = 50_000
    max_counter: int = 1_000
    # Simulation start time (fixed for reproducibility)
    start_time_unix: int = 1_700_000_000


# ---------------------------------------------------------------------------
# In-process reconciliation engine
# ---------------------------------------------------------------------------


@dataclass
class _RecordedTransaction:
    """Internal backend record."""

    envelope: PaymentEnvelope
    state: TransactionState


class ReconciliationEngine:
    """In-process reconciliation engine mirroring RECONCILIATION_SPEC.md §3.

    Used by the simulator without a real backend server. The same 8-step flow
    is followed so that simulator results are meaningful evidence.
    """

    def __init__(self) -> None:
        self._records: dict[str, _RecordedTransaction] = {}
        self._credentials: dict[str, Credential] = {}
        self._credential_counters: dict[str, dict[int, str]] = {}

    def register_credential(self, cred: Credential) -> None:
        self._credentials[cred.credential_id] = cred

    def reconcile(self, envelope: PaymentEnvelope, now_unix: int) -> tuple[TransactionState, str]:
        """Run the normative ingestion flow.

        Returns (resulting_state, reason).
        """
        # Step 1: schema validate
        if envelope.protocol_version != PROTOCOL_VERSION:
            return TransactionState.REJECTED, "wrong protocol version"
        if len(envelope.nonce) != 16:
            return TransactionState.REJECTED, "nonce must be 16 bytes"
        if envelope.signature_bytes is None or len(envelope.signature_bytes) != 64:
            return TransactionState.REJECTED, "signature must be 64 bytes"
        if envelope.counter < 1:
            return TransactionState.REJECTED, "counter must be >= 1"

        # Step 2: look up credential
        cred = self._credentials.get(envelope.credential_id)
        if cred is None:
            return TransactionState.REJECTED, "unknown credential"

        # Step 3: verify signature (server-side re-derivation)
        try:
            verify_envelope(envelope, envelope.signature_bytes, cred.public_key_bytes)
        except InvalidSignature:
            return TransactionState.REJECTED, "signature verification failed"

        # Step 4: check credential status
        if not cred.is_active(now_unix):
            return TransactionState.REJECTED, f"credential not active (state={cred.state.value})"

        # Step 5: idempotency / conflict
        existing = self._records.get(envelope.tx_id)
        if existing is not None:
            if (
                existing.envelope.counter == envelope.counter
                and existing.envelope.amount.amount_minor == envelope.amount.amount_minor
                and existing.envelope.nonce == envelope.nonce
            ):
                return TransactionState.RECONCILED, "already_known"
            # Conflict: same tx_id, different content
            existing.state = TransactionState.CONFLICT
            return TransactionState.CONFLICT, "conflicting evidence"

        # Check duplicate counter for same credential with different tx_id (double spend)
        used_counters = self._credential_counters.setdefault(envelope.credential_id, {})
        if envelope.counter in used_counters and used_counters[envelope.counter] != envelope.tx_id:
            return (
                TransactionState.CONFLICT,
                "conflicting evidence: duplicate counter for credential",
            )

        # Step 6: counter/policy
        if envelope.counter > cred.max_counter:
            return TransactionState.REJECTED, "counter exceeds credential maximum"
        if envelope.amount.amount_minor > cred.max_value_per_tx_minor:
            return TransactionState.REJECTED, "amount exceeds per-tx limit"

        # Step 7: persist
        self._records[envelope.tx_id] = _RecordedTransaction(
            envelope=envelope, state=TransactionState.RECONCILED
        )
        used_counters[envelope.counter] = envelope.tx_id
        return TransactionState.RECONCILED, "accepted"

    @property
    def total_reconciled(self) -> int:
        return sum(1 for r in self._records.values() if r.state == TransactionState.RECONCILED)

    @property
    def total_conflicts(self) -> int:
        return sum(1 for r in self._records.values() if r.state == TransactionState.CONFLICT)


# ---------------------------------------------------------------------------
# Simulation result
# ---------------------------------------------------------------------------


@dataclass
class SimulationResult:
    """Reproducibility-complete simulation result.

    Every field needed to reproduce or audit the run is included.
    A screenshot or summary alone is NOT sufficient evidence.
    """

    run_id: str
    scenario_id: str
    seed: int
    protocol_version: int
    start_time_unix: int

    total_transactions: int = 0
    total_signed: int = 0
    total_transferred: int = 0
    total_locally_verified: int = 0
    total_reconciled: int = 0
    total_rejected: int = 0
    total_conflicts: int = 0
    total_transport_losses: int = 0
    total_transport_duplicates: int = 0

    event_log: list[SimEvent] = field(default_factory=list)


# ---------------------------------------------------------------------------
# Simulator engine
# ---------------------------------------------------------------------------


class Simulator:
    """Drives a simulation run from a ScenarioConfig.

    All randomness is isolated in self.rng so the run is reproducible
    from (scenario_id, seed).
    """

    def __init__(self, config: ScenarioConfig) -> None:
        self.config = config
        self.risk_model = RuleBasedRiskModel()
        self.rng = random.Random(config.seed)
        self._reconciler = ReconciliationEngine()
        self._event_log: list[SimEvent] = []
        self._sim_time = config.start_time_unix

    def _emit(self, kind: SimEventKind, **kwargs: object) -> None:
        self._event_log.append(
            SimEvent(kind=kind, sim_time=self._sim_time, **kwargs)  # type: ignore[arg-type]
        )

    def _tick(self, seconds: int = 1) -> None:
        self._sim_time += seconds

    # ------------------------------------------------------------------
    # Participant creation
    # ------------------------------------------------------------------

    def _make_payer(self) -> PayerDevice:
        seed = bytes(self.rng.randint(0, 255) for _ in range(32))
        private_seed, public_key = generate_keypair()
        # Override with deterministic seed for reproducibility
        from resilientpay_sim.domain.crypto import keypair_from_seed

        private_seed, public_key = keypair_from_seed(seed)

        cred = Credential(
            credential_id=str(uuid.UUID(int=self.rng.getrandbits(128))),
            subject_key_id=str(uuid.UUID(int=self.rng.getrandbits(128))),
            public_key_bytes=public_key,
            issued_at_unix=self._sim_time,
            expires_at_unix=self._sim_time + self.config.credential_valid_seconds,
            max_value_per_tx_minor=self.config.max_value_per_tx_minor,
            max_counter=self.config.max_counter,
            state=CredentialState.ACTIVE,
        )
        self._reconciler.register_credential(cred)

        return PayerDevice(
            device_id=str(uuid.UUID(int=self.rng.getrandbits(128))),
            user_id=str(uuid.UUID(int=self.rng.getrandbits(128))),
            credential=cred,
            private_key_bytes=private_seed,
            connectivity=self.config.connectivity,
        )

    def _make_merchant(self) -> Merchant:
        return Merchant(
            merchant_id=str(uuid.UUID(int=self.rng.getrandbits(128))),
            connectivity=self.config.connectivity,
        )

    # ------------------------------------------------------------------
    # Single transaction flow
    # ------------------------------------------------------------------

    def _run_transaction(
        self,
        payer: PayerDevice,
        merchant: Merchant,
        channel: TransportChannel,
    ) -> tuple[TransactionState, bool]:
        """Execute one full payment transaction from initiation to reconciliation.

        Returns (final_state, transport_delivered).
        """
        assert payer.credential is not None
        assert payer.private_key_bytes is not None

        amount_minor = self.rng.randint(1, min(1_000, self.config.max_value_per_tx_minor))
        nonce = bytes(self.rng.randint(0, 255) for _ in range(16))

        envelope = PaymentEnvelope(
            protocol_version=PROTOCOL_VERSION,
            tx_id=str(uuid.UUID(int=self.rng.getrandbits(128))),
            credential_id=payer.credential.credential_id,
            payer_key_id=payer.credential.subject_key_id,
            merchant_id=merchant.merchant_id,
            amount=Money(amount_minor=amount_minor, currency="INR"),
            counter=payer.next_counter(),
            nonce=nonce,
            created_at_unix=self._sim_time,
            expires_at_unix=self._sim_time + 3_600,
            risk_class=None,
        )
        # Classify risk (advisory only per ADR-010)
        envelope = attr.evolve(envelope, risk_class=self.risk_model.classify(envelope).value)

        self._emit(
            SimEventKind.PAYMENT_INITIATED,
            tx_id=envelope.tx_id,
            device_id=payer.device_id,
            merchant_id=merchant.merchant_id,
        )

        # Sign
        sig = sign_envelope(envelope, payer.private_key_bytes)
        # Use attr.evolve() — slots=True means vars() is unavailable
        envelope = attr.evolve(envelope, signature_bytes=sig)

        self._emit(SimEventKind.ENVELOPE_SIGNED, tx_id=envelope.tx_id, device_id=payer.device_id)

        # Transport
        delivered = channel.send(envelope, self._sim_time)
        if not delivered:
            self._emit(
                SimEventKind.TRANSPORT_FAILED,
                tx_id=envelope.tx_id,
                device_id=payer.device_id,
                detail=f"transport={channel.kind.value} lost",
            )
            return TransactionState.SYNC_PENDING, False

        self._emit(
            SimEventKind.ENVELOPE_TRANSFERRED,
            tx_id=envelope.tx_id,
            device_id=payer.device_id,
            merchant_id=merchant.merchant_id,
        )

        # Merchant locally verifies
        try:
            verify_envelope(envelope, sig, payer.credential.public_key_bytes)
        except InvalidSignature:
            self._emit(
                SimEventKind.FAULT_INJECTED,
                tx_id=envelope.tx_id,
                detail="local verification failed",
            )
            return TransactionState.REJECTED, True

        self._emit(
            SimEventKind.LOCALLY_VERIFIED, tx_id=envelope.tx_id, merchant_id=merchant.merchant_id
        )
        self._emit(SimEventKind.LOCALLY_RECORDED, tx_id=envelope.tx_id)
        merchant.received_envelopes.append(envelope)

        # Reconcile (in-process)
        state, reason = self._reconciler.reconcile(envelope, self._sim_time)
        kind = (
            SimEventKind.RECONCILED
            if state == TransactionState.RECONCILED
            else SimEventKind.CONFLICT
            if state == TransactionState.CONFLICT
            else SimEventKind.REJECTED
        )
        self._emit(kind, tx_id=envelope.tx_id, detail=reason)
        return state, True

    # ------------------------------------------------------------------
    # Run
    # ------------------------------------------------------------------

    def run(self) -> SimulationResult:
        """Execute the full scenario and return a reproducibility-complete result."""
        cfg = self.config
        result = SimulationResult(
            run_id=str(uuid.UUID(int=self.rng.getrandbits(128))),
            scenario_id=cfg.scenario_id,
            seed=cfg.seed,
            protocol_version=PROTOCOL_VERSION,
            start_time_unix=cfg.start_time_unix,
        )

        payers = [self._make_payer() for _ in range(cfg.num_payers)]
        merchants = [self._make_merchant() for _ in range(cfg.num_merchants)]

        channel = TransportChannel(
            kind=cfg.transport_kind,
            fault_profile=cfg.fault_profile,
            rng=self.rng,
        )

        for _ in range(cfg.num_transactions):
            payer = self.rng.choice(payers)
            merchant = self.rng.choice(merchants)
            self._tick(self.rng.randint(1, 60))  # random inter-arrival time

            state, delivered = self._run_transaction(payer, merchant, channel)
            result.total_transactions += 1
            result.total_signed += 1

            if delivered:
                result.total_transferred += 1
                result.total_locally_verified += 1
            else:
                result.total_transport_losses += 1

            if state == TransactionState.RECONCILED:
                result.total_reconciled += 1
            elif state == TransactionState.REJECTED:
                result.total_rejected += 1
            elif state == TransactionState.CONFLICT:
                result.total_conflicts += 1

        # Count channel duplicates
        result.total_transport_duplicates = sum(
            1 for ev in channel.events if ev.get("type") == "TRANSPORT_DUPLICATE"
        )

        result.event_log = list(self._event_log)
        return result
