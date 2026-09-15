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
    """

    kind: TransportKind
    fault_profile: FaultProfile = field(default_factory=FaultProfile)
    rng: random.Random = field(default_factory=random.Random)

    # Delivered messages (received by recipient after fault injection)
    delivered: list[PaymentEnvelope] = field(default_factory=list)
    # Events recorded for test inspection
    events: list[dict[str, Any]] = field(default_factory=list)

    def send(self, envelope: PaymentEnvelope, sim_time: int) -> bool:
        """Attempt to send an envelope through the channel.

        Returns True if at least one copy was delivered; False if all copies
        were lost. This return value MUST NOT be used to determine payment
        validity — it is only for transport-layer telemetry.
        """
        fp = self.fault_profile

        # 1. Loss
        if fp.loss_probability > 0 and self.rng.random() < fp.loss_probability:
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
            self.events.append(
                {
                    "type": "TRANSPORT_DELIVERED",
                    "tx_id": envelope.tx_id,
                    "sim_time": effective_time,
                    "transport": self.kind.value,
                }
            )

        return True

    def clear(self) -> None:
        """Reset the channel's delivered queue (use between simulation steps)."""
        self.delivered.clear()
        self.events.clear()
