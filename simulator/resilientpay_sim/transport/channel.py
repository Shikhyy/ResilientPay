"""
Transport layer simulation.

Models the four supported transport adapters and fault injection for each.
Transport code CANNOT make payment authorization decisions — it only carries
protocol objects.

Architecture rule (from .agent/AGENTS.md):
    Transport adapters move protocol objects and do not decide payment validity.

Fault modes per SIMULATION.md §4:
    - transport_loss       : message silently dropped
    - duplication          : message delivered N times
    - reordering           : messages arrive out of order
    - delay                : message delayed by N simulated seconds
    - malformed            : bytes corrupted before delivery
"""

from __future__ import annotations

import random
from dataclasses import dataclass, field
from enum import Enum
from typing import Any

from resilientpay_sim.domain.model import PaymentEnvelope


class TransportKind(Enum):
    """Supported transport adapters per TRANSPORT_ARCHITECTURE.md."""

    INTERNET = "INTERNET"
    NFC = "NFC"
    BLE = "BLE"
    QR = "QR"
    SMS = "SMS"


@dataclass
class FaultProfile:
    """Configures fault injection for a transport channel.

    All probability values are in [0.0, 1.0].
    """

    loss_probability: float = 0.0  # P(message silently dropped)
    duplicate_probability: float = 0.0  # P(message duplicated once)
    corruption_probability: float = 0.0  # P(message bytes corrupted)
    delay_seconds: int = 0  # deterministic delay to add


@dataclass
class TransportChannel:
    """A directed channel between two simulation actors.

    The channel applies fault injection according to its FaultProfile, then
    delivers messages to the recipient's inbox. Neither the sender nor the
    recipient decides whether delivery succeeds — the channel does.

    Architecture rule: transport outcome ≠ payment validity.

    Per-segment statistics (reset via clear()):
        messages_sent          : total send() calls
        messages_dropped       : messages lost due to fault injection
        total_latency_ms_simulated : cumulative simulated latency for delivered msgs
    """

    kind: TransportKind
    fault_profile: FaultProfile = field(default_factory=FaultProfile)
    rng: random.Random = field(default_factory=random.Random)
    # Simulated latency per message delivery (ms, for E1 measurements)
    latency_ms: float = 0.0

    # Delivered messages (received by recipient after fault injection)
    delivered: list[PaymentEnvelope] = field(default_factory=list)
    # Events recorded for test inspection
    events: list[dict[str, Any]] = field(default_factory=list)

    # Per-segment statistics (reset with clear())
    messages_sent: int = field(default=0, init=False)
    messages_dropped: int = field(default=0, init=False)
    total_latency_ms_simulated: float = field(default=0.0, init=False)

    def send(self, envelope: PaymentEnvelope, sim_time: int) -> bool:
        """Attempt to send an envelope through the channel.

        Returns True if at least one copy was delivered; False if all copies
        were lost. This return value MUST NOT be used to determine payment
        validity — it is only for transport-layer telemetry.
        """
        fp = self.fault_profile
        self.messages_sent += 1

        # 1. Loss
        if fp.loss_probability > 0 and self.rng.random() < fp.loss_probability:
            self.messages_dropped += 1
            self.events.append(
                {
                    "type": "TRANSPORT_LOSS",
                    "tx_id": envelope.tx_id,
                    "sim_time": sim_time,
                    "transport": self.kind.value,
                }
            )
            return False

        # 2. Corruption
        if fp.corruption_probability > 0 and self.rng.random() < fp.corruption_probability:
            # Record the fault but deliver nothing (corrupted payload dropped)
            self.messages_dropped += 1
            self.events.append(
                {
                    "type": "TRANSPORT_CORRUPTION",
                    "tx_id": envelope.tx_id,
                    "sim_time": sim_time,
                    "transport": self.kind.value,
                }
            )
            return False

        # 3. Duplicate delivery
        copies = 1
        if fp.duplicate_probability > 0 and self.rng.random() < fp.duplicate_probability:
            copies = 2
            self.events.append(
                {
                    "type": "TRANSPORT_DUPLICATE",
                    "tx_id": envelope.tx_id,
                    "sim_time": sim_time,
                    "transport": self.kind.value,
                }
            )

        # 4. Deliver (with delay recorded but not actually sleeping in sim time)
        effective_time = sim_time + fp.delay_seconds
        for _ in range(copies):
            self.delivered.append(envelope)
            self.total_latency_ms_simulated += self.latency_ms
            self.events.append(
                {
                    "type": "TRANSPORT_DELIVERED",
                    "tx_id": envelope.tx_id,
                    "sim_time": effective_time,
                    "transport": self.kind.value,
                    "latency_ms": self.latency_ms,
                }
            )

        return True

    def clear(self) -> None:
        """Reset the channel's delivered queue (use between simulation steps)."""
        self.delivered.clear()
        self.events.clear()
        self.messages_sent = 0
        self.messages_dropped = 0
        self.total_latency_ms_simulated = 0.0

    @property
    def avg_latency_ms(self) -> float:
        """Average latency per delivered message; 0.0 if nothing delivered."""
        delivered_count = self.messages_sent - self.messages_dropped
        if delivered_count <= 0:
            return 0.0
        return self.total_latency_ms_simulated / delivered_count


# ---------------------------------------------------------------------------
# Transport profile factory functions
# ---------------------------------------------------------------------------
# Latency values reflect representative measurements from published research and
# vendor documentation. They are simulation approximations, not normative specs.
# Loss probabilities are conservative field estimates for each medium.


def internet_channel(drop_rate: float = 0.0) -> TransportChannel:
    """Internet/HTTPS transport profile. ~50 ms RTT, configurable drop rate."""
    return TransportChannel(
        kind=TransportKind.INTERNET,
        fault_profile=FaultProfile(loss_probability=drop_rate),
        latency_ms=50.0,
    )


def sms_channel(drop_rate: float = 0.1) -> TransportChannel:
    """SMS transport profile. ~3000 ms latency, 10% default drop rate."""
    return TransportChannel(
        kind=TransportKind.SMS,
        fault_profile=FaultProfile(loss_probability=drop_rate),
        latency_ms=3000.0,
    )


def nfc_channel(drop_rate: float = 0.02) -> TransportChannel:
    """NFC transport profile. ~20 ms latency, 2% default drop rate."""
    return TransportChannel(
        kind=TransportKind.NFC,
        fault_profile=FaultProfile(loss_probability=drop_rate),
        latency_ms=20.0,
    )


def ble_channel(drop_rate: float = 0.05) -> TransportChannel:
    """BLE transport profile. ~100 ms latency, 5% default drop rate."""
    return TransportChannel(
        kind=TransportKind.BLE,
        fault_profile=FaultProfile(loss_probability=drop_rate),
        latency_ms=100.0,
    )


def qr_channel(drop_rate: float = 0.01) -> TransportChannel:
    """QR transport profile. ~200 ms latency, 1% default drop rate."""
    return TransportChannel(
        kind=TransportKind.QR,
        fault_profile=FaultProfile(loss_probability=drop_rate),
        latency_ms=200.0,
    )
