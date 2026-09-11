"""
ResilientPay Simulator — domain model.

This module defines the core simulation entities as immutable attrs classes.
All monetary values are integer minor units (no float). UUIDs are the str
representation of 128-bit identifiers.

Protocol reference:
    docs/07-research-experiments/SIMULATION.md §3 (simulation entities)
    docs/05-protocol/PAYMENT_PROTOCOL.md
    docs/05-protocol/OFFLINE_CREDENTIAL_SPEC.md
"""
from __future__ import annotations

import uuid
from enum import Enum
from typing import Optional
import attr

# ---------------------------------------------------------------------------
# Money — integer minor units only
# ---------------------------------------------------------------------------

MAX_AMOUNT_MINOR: int = 100_000_000  # ₹1,000,000 in paise (per SDK)


@attr.s(frozen=True, auto_attribs=True, slots=True)
class Money:
    """Monetary amount as integer minor units + explicit currency.

    Security rule: NEVER use float for monetary values.
    """
    amount_minor: int = attr.ib()
    currency: str = attr.ib()

    @amount_minor.validator  # type: ignore[misc]
    def _validate_amount(self, _attribute: attr.Attribute, value: int) -> None:  # type: ignore[type-arg]
        if not isinstance(value, int):
            raise TypeError(f"amount_minor must be int, got {type(value)}")
        if value < 0:
            raise ValueError(f"amount_minor must be non-negative, got {value}")
        if value > MAX_AMOUNT_MINOR:
            raise ValueError(f"amount_minor {value} exceeds maximum {MAX_AMOUNT_MINOR}")

    @currency.validator  # type: ignore[misc]
    def _validate_currency(self, _attribute: attr.Attribute, value: str) -> None:  # type: ignore[type-arg]
        if not (1 <= len(value) <= 8):
            raise ValueError(f"currency must be 1–8 chars, got {value!r}")


# ---------------------------------------------------------------------------
# Connectivity state (C0–C3 per architecture)
# ---------------------------------------------------------------------------

class ConnectivityState(Enum):
    """Device connectivity level.

    C0 = no connectivity at all
    C1 = can communicate via peer-to-peer (NFC/BLE/QR/SMS) only
    C2 = intermittent internet (high packet loss / latency)
    C3 = full internet connectivity
    """
    C0 = "C0"  # fully offline
    C1 = "C1"  # peer-to-peer only
    C2 = "C2"  # intermittent internet
    C3 = "C3"  # full connectivity


# ---------------------------------------------------------------------------
# Transaction state (mirrors SDK state_machine.rs)
# ---------------------------------------------------------------------------

class TransactionState(Enum):
    CREATED = "CREATED"
    VALIDATING = "VALIDATING"
    AUTHORIZED = "AUTHORIZED"
    SIGNED = "SIGNED"
    TRANSFERRED = "TRANSFERRED"
    RECEIVED = "RECEIVED"
    LOCALLY_VERIFIED = "LOCALLY_VERIFIED"
    LOCALLY_RECORDED = "LOCALLY_RECORDED"
    SYNC_PENDING = "SYNC_PENDING"
    RECONCILED = "RECONCILED"
    REJECTED = "REJECTED"
    CONFLICT = "CONFLICT"

    @property
    def is_terminal(self) -> bool:
        return self in (
            TransactionState.RECONCILED,
            TransactionState.REJECTED,
            TransactionState.CONFLICT,
        )


# ---------------------------------------------------------------------------
# Credential lifecycle
# ---------------------------------------------------------------------------

class CredentialState(Enum):
    REQUESTED = "REQUESTED"
    ISSUED = "ISSUED"
    ACTIVE = "ACTIVE"
    SUSPENDED = "SUSPENDED"
    REVOKED = "REVOKED"
    EXPIRED = "EXPIRED"


@attr.s(frozen=True, auto_attribs=True, slots=True)
class Credential:
    """Offline authorization credential for a payer device."""
    # Mandatory fields first (no defaults)
    public_key_bytes: bytes = attr.ib()  # 32 bytes Ed25519 verifying key
    issued_at_unix: int = attr.ib()
    expires_at_unix: int = attr.ib()
    # Fields with defaults
    credential_id: str = attr.ib(factory=lambda: str(uuid.uuid4()))
    subject_key_id: str = attr.ib(factory=lambda: str(uuid.uuid4()))
    max_value_per_tx_minor: int = attr.ib(default=50_000)
    max_value_outstanding_minor: int = attr.ib(default=200_000)
    max_counter: int = attr.ib(default=1_000)
    state: CredentialState = attr.ib(default=CredentialState.ACTIVE)


    def is_active(self, now_unix: int) -> bool:
        return self.state == CredentialState.ACTIVE and now_unix < self.expires_at_unix


# ---------------------------------------------------------------------------
# Payment envelope (mirrors SDK PaymentEnvelopeCore)
# ---------------------------------------------------------------------------

@attr.s(frozen=True, auto_attribs=True, slots=True)
class PaymentEnvelope:
    """The signed protocol envelope. All 13 fields match PAYMENT_PROTOCOL.md §2."""
    # Mandatory fields — no defaults
    protocol_version: int = attr.ib()
    credential_id: str = attr.ib()
    payer_key_id: str = attr.ib()
    merchant_id: str = attr.ib()
    amount: Money = attr.ib()
    counter: int = attr.ib()
    nonce: bytes = attr.ib()           # 16 bytes
    created_at_unix: int = attr.ib()
    expires_at_unix: int = attr.ib()
    # Optional / auto-generated fields (must come after mandatory)
    tx_id: str = attr.ib(factory=lambda: str(uuid.uuid4()))
    previous_event_hash: Optional[bytes] = attr.ib(default=None)   # 32 bytes or None
    risk_class: Optional[str] = attr.ib(default=None)
    # Signature is separate (not part of signed content)
    signature_bytes: Optional[bytes] = attr.ib(default=None)   # 64 bytes



# ---------------------------------------------------------------------------
# Simulation actors
# ---------------------------------------------------------------------------

@attr.s(auto_attribs=True)
class PayerDevice:
    """A simulated payer device."""
    device_id: str = attr.ib(factory=lambda: str(uuid.uuid4()))
    user_id: str = attr.ib(factory=lambda: str(uuid.uuid4()))
    credential: Optional[Credential] = attr.ib(default=None)
    private_key_bytes: Optional[bytes] = attr.ib(default=None)  # 32-byte Ed25519 seed
    counter: int = attr.ib(default=0)
    connectivity: ConnectivityState = attr.ib(default=ConnectivityState.C3)
    # Pending sync queue (transactions awaiting reconciliation)
    sync_queue: list[PaymentEnvelope] = attr.ib(factory=list)

    def next_counter(self) -> int:
        self.counter += 1
        return self.counter


@attr.s(auto_attribs=True)
class Merchant:
    """A simulated merchant node."""
    merchant_id: str = attr.ib(factory=lambda: str(uuid.uuid4()))
    name: str = attr.ib(default="Merchant")
    connectivity: ConnectivityState = attr.ib(default=ConnectivityState.C3)
    received_envelopes: list[PaymentEnvelope] = attr.ib(factory=list)


# ---------------------------------------------------------------------------
# Simulation event log (immutable records)
# ---------------------------------------------------------------------------

class SimEventKind(Enum):
    PAYMENT_INITIATED = "PAYMENT_INITIATED"
    ENVELOPE_SIGNED = "ENVELOPE_SIGNED"
    ENVELOPE_TRANSFERRED = "ENVELOPE_TRANSFERRED"
    ENVELOPE_RECEIVED = "ENVELOPE_RECEIVED"
    LOCALLY_VERIFIED = "LOCALLY_VERIFIED"
    LOCALLY_RECORDED = "LOCALLY_RECORDED"
    SYNC_SUBMITTED = "SYNC_SUBMITTED"
    RECONCILED = "RECONCILED"
    REJECTED = "REJECTED"
    CONFLICT = "CONFLICT"
    TRANSPORT_FAILED = "TRANSPORT_FAILED"
    FAULT_INJECTED = "FAULT_INJECTED"


@attr.s(frozen=True, auto_attribs=True, slots=True)
class SimEvent:
    """An immutable record of a simulation event."""
    kind: SimEventKind = attr.ib()
    sim_time: int = attr.ib()   # simulated unix timestamp
    tx_id: Optional[str] = attr.ib(default=None)
    device_id: Optional[str] = attr.ib(default=None)
    merchant_id: Optional[str] = attr.ib(default=None)
    detail: Optional[str] = attr.ib(default=None)
