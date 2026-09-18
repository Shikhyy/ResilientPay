#!/usr/bin/env python3
"""
E1 — Transport Performance Experiment
======================================
For each of the five supported transport profiles (internet, sms, nfc, ble, qr)
run 200 transactions with a seeded SimulationEngine and collect:

    - success_rate          : fraction that reached RECONCILED state
    - avg_latency_ms        : simulated transport latency per delivered message
    - cbor_payload_bytes    : CBOR envelope size (averaged across transactions)
    - sms_segments_avg      : estimated number of 153-byte SMS segments required
    - transport_loss_rate   : fraction of messages dropped by the channel

Results written to simulator/results/e1_transport_performance.json with
reproducibility metadata (timestamp, git commit, seed, protocol_version).

Reference: docs/07-research-experiments/EXPERIMENT_PLAN_DETAIL.md E1
"""

from __future__ import annotations

import json
import math
import subprocess
import sys
import time
from pathlib import Path

_SIMULATOR_ROOT = Path(__file__).parent.parent
sys.path.insert(0, str(_SIMULATOR_ROOT))

import attr
from resilientpay_sim.domain.crypto import encode_envelope_cbor, keypair_from_seed, sign_envelope
from resilientpay_sim.domain.model import (
    ConnectivityState,
    Credential,
    CredentialState,
    Money,
    PaymentEnvelope,
)
from resilientpay_sim.engine import PROTOCOL_VERSION, ScenarioConfig, Simulator
from resilientpay_sim.risk.model import RuleBasedRiskModel
from resilientpay_sim.transport.channel import (
    FaultProfile,
    TransportKind,
    ble_channel,
    internet_channel,
    nfc_channel,
    qr_channel,
    sms_channel,
)

SEED = 42
NUM_TRANSACTIONS = 200
SMS_SEGMENT_BYTES = 153
PROTOCOL_VERSION_STR = "1.0"

TRANSPORT_PROFILES = [
    ("internet", TransportKind.INTERNET, internet_channel),
    ("sms", TransportKind.SMS, sms_channel),
    ("nfc", TransportKind.NFC, nfc_channel),
    ("ble", TransportKind.BLE, ble_channel),
    ("qr", TransportKind.QR, qr_channel),
]


def _git_commit() -> str:
    try:
        return subprocess.check_output(
            ["git", "rev-parse", "HEAD"],
            cwd=str(_SIMULATOR_ROOT),
            stderr=subprocess.DEVNULL,
        ).decode().strip()
    except Exception:
        return "unknown"


def _sample_cbor_size() -> float:
    """Sample average CBOR payload size over 20 envelopes (seed=SEED)."""
    import random, uuid as uuid_mod
    rng = random.Random(SEED)
    risk_model = RuleBasedRiskModel()
    sim_time = 1_700_000_000
    sizes: list[int] = []
    for _ in range(20):
        seed_bytes = bytes(rng.randint(0, 255) for _ in range(32))
        private_seed, public_key = keypair_from_seed(seed_bytes)
        cred = Credential(
            public_key_bytes=public_key,
            issued_at_unix=sim_time,
            expires_at_unix=sim_time + 3600,
        )
        amount_minor = rng.randint(1, 1000)
        nonce = bytes(rng.randint(0, 255) for _ in range(16))
        envelope = PaymentEnvelope(
            protocol_version=PROTOCOL_VERSION,
            tx_id=str(uuid_mod.UUID(int=rng.getrandbits(128))),
            credential_id=cred.credential_id,
            payer_key_id=cred.subject_key_id,
            merchant_id=str(uuid_mod.UUID(int=rng.getrandbits(128))),
            amount=Money(amount_minor=amount_minor, currency="INR"),
            counter=1,
            nonce=nonce,
            created_at_unix=sim_time,
            expires_at_unix=sim_time + 3600,
            risk_class=None,
        )
        envelope = attr.evolve(envelope, risk_class=risk_model.classify(envelope).value)
        sig = sign_envelope(envelope, private_seed)
        envelope = attr.evolve(envelope, signature_bytes=sig)
        sizes.append(len(encode_envelope_cbor(envelope)))
    return sum(sizes) / len(sizes)


def run_transport(name: str, kind: TransportKind, factory) -> dict:
    channel = factory()
    config = ScenarioConfig(
        scenario_id=f"e1_{name}_v1",
        seed=SEED,
        num_payers=20,
        num_merchants=5,
        num_transactions=NUM_TRANSACTIONS,
        connectivity=ConnectivityState.C3,
        transport_kind=kind,
        fault_profile=channel.fault_profile,
        credential_valid_seconds=86_400,
    )
    result = Simulator(config).run()

    total = result.total_transactions
    success_rate = result.total_reconciled / total if total else 0.0
    loss_rate = result.total_transport_losses / total if total else 0.0
    avg_latency_ms = channel.latency_ms * (1.0 - loss_rate)

    avg_cbor = _sample_cbor_size()
    sms_segments = math.ceil(avg_cbor / SMS_SEGMENT_BYTES)

    return {
        "transport": name,
        "num_transactions": total,
        "total_reconciled": result.total_reconciled,
        "total_transport_losses": result.total_transport_losses,
        "success_rate": round(success_rate, 4),
        "transport_loss_rate": round(loss_rate, 4),
        "avg_latency_ms": round(avg_latency_ms, 2),
        "channel_latency_ms": channel.latency_ms,
        "cbor_payload_bytes_avg": round(avg_cbor, 1),
        "sms_segments_avg": sms_segments,
    }


def main() -> None:
    results = []
    for name, kind, factory in TRANSPORT_PROFILES:
        r = run_transport(name, kind, factory)
        results.append(r)

    out_dir = _SIMULATOR_ROOT / "results"
    out_dir.mkdir(parents=True, exist_ok=True)
    out_path = out_dir / "e1_transport_performance.json"

    report = {
        "experiment": "E1_transport_performance",
        "schema_version": 1,
        "metadata": {
            "timestamp_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "git_commit": _git_commit(),
            "seed": SEED,
            "num_transactions_per_transport": NUM_TRANSACTIONS,
            "protocol_version": PROTOCOL_VERSION_STR,
            "sms_segment_bytes": SMS_SEGMENT_BYTES,
        },
        "results": results,
    }

    with open(out_path, "w") as f:
        json.dump(report, f, indent=2)

    header = f"{'Transport':<10} {'Success%':>9} {'Loss%':>7} {'LatencyMs':>10} {'CBOR B':>8} {'SMS Seg':>8}"
    sep = "=" * len(header)
    print(sep)
    print("E1 Transport Performance (seed=42, 200 txns each)")
    print(sep)
    print(header)
    print("-" * len(header))
    for r in results:
        print(
            f"{r['transport']:<10}"
            f" {r['success_rate']*100:>8.1f}%"
            f" {r['transport_loss_rate']*100:>6.1f}%"
            f" {r['avg_latency_ms']:>10.1f}"
            f" {r['cbor_payload_bytes_avg']:>8.1f}"
            f" {r['sms_segments_avg']:>8}"
        )
    print(sep)
    print(f"\nResults written to: {out_path}")


if __name__ == "__main__":
    main()
