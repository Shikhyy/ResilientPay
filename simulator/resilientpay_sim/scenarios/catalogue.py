"""
Built-in scenario definitions for the ResilientPay simulator.

Each scenario is a named, versioned ScenarioConfig that can be run deterministically
from its seed. Scenarios are research evidence — they should be retained alongside
simulation results.

Scenario catalogue:
    happy_path_internet   — baseline: full connectivity, no faults
    partial_loss          — 30% transport loss, internet transport
    fully_offline         — C0 connectivity, no reconciliation possible
    replay_attack         — duplicate submissions to test idempotency
    high_volume           — 1000 txns, baseline connectivity stress test
"""
from __future__ import annotations

from resilientpay_sim.engine import ScenarioConfig
from resilientpay_sim.domain.model import ConnectivityState
from resilientpay_sim.transport.channel import FaultProfile, TransportKind


def happy_path_internet(seed: int = 42) -> ScenarioConfig:
    """Baseline scenario: full internet connectivity, no faults.

    Used to establish the expected success rate under ideal conditions.
    """
    return ScenarioConfig(
        scenario_id="happy_path_internet_v1",
        seed=seed,
        num_payers=10,
        num_merchants=3,
        num_transactions=20,
        connectivity=ConnectivityState.C3,
        transport_kind=TransportKind.INTERNET,
        fault_profile=FaultProfile(),  # no faults
    )


def partial_loss_internet(seed: int = 42) -> ScenarioConfig:
    """30% transport loss — tests sync queue and retry behaviour.

    Transactions that don't deliver via transport enter SYNC_PENDING.
    The reconciliation rate should be ~70% of signed transactions.
    """
    return ScenarioConfig(
        scenario_id="partial_loss_internet_v1",
        seed=seed,
        num_payers=10,
        num_merchants=3,
        num_transactions=20,
        connectivity=ConnectivityState.C2,
        transport_kind=TransportKind.INTERNET,
        fault_profile=FaultProfile(loss_probability=0.30),
    )


def duplicate_delivery(seed: int = 42) -> ScenarioConfig:
    """100% duplicate delivery — tests idempotency (ALREADY_KNOWN).

    Every transaction is delivered twice. The reconciliation engine should
    accept the first delivery and return ALREADY_KNOWN for the second.
    Conflict count must be 0 (duplicates are not conflicts).
    """
    return ScenarioConfig(
        scenario_id="duplicate_delivery_v1",
        seed=seed,
        num_payers=5,
        num_merchants=2,
        num_transactions=10,
        connectivity=ConnectivityState.C3,
        transport_kind=TransportKind.INTERNET,
        fault_profile=FaultProfile(duplicate_probability=1.0),
    )


def fully_offline(seed: int = 42) -> ScenarioConfig:
    """C0 connectivity — all transport fails, all transactions SYNC_PENDING.

    Tests that the system does not lose transactions when offline.
    """
    return ScenarioConfig(
        scenario_id="fully_offline_v1",
        seed=seed,
        num_payers=5,
        num_merchants=2,
        num_transactions=10,
        connectivity=ConnectivityState.C0,
        transport_kind=TransportKind.INTERNET,
        fault_profile=FaultProfile(loss_probability=1.0),  # 100% loss
    )


def high_volume_baseline(seed: int = 42) -> ScenarioConfig:
    """Stress test: 1000 transactions, full connectivity.

    Validates that the simulator and reconciliation engine scale linearly.
    """
    return ScenarioConfig(
        scenario_id="high_volume_baseline_v1",
        seed=seed,
        num_payers=100,
        num_merchants=20,
        num_transactions=1_000,
        connectivity=ConnectivityState.C3,
        transport_kind=TransportKind.INTERNET,
        fault_profile=FaultProfile(),
    )


def nfc_partial_loss(seed: int = 42) -> ScenarioConfig:
    """NFC transport with 20% loss — models near-field payment failures."""
    return ScenarioConfig(
        scenario_id="nfc_partial_loss_v1",
        seed=seed,
        num_payers=5,
        num_merchants=2,
        num_transactions=20,
        connectivity=ConnectivityState.C1,
        transport_kind=TransportKind.NFC,
        fault_profile=FaultProfile(loss_probability=0.20),
    )


# Registry for CLI and test use
ALL_SCENARIOS = {
    "happy_path_internet": happy_path_internet,
    "partial_loss_internet": partial_loss_internet,
    "duplicate_delivery": duplicate_delivery,
    "fully_offline": fully_offline,
    "high_volume_baseline": high_volume_baseline,
    "nfc_partial_loss": nfc_partial_loss,
}
