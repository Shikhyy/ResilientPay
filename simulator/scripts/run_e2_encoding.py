#!/usr/bin/env python3
"""
E2 — Encoding Comparison Experiment
===================================
Compares canonical CBOR and compact JSON across representative transaction configurations:
    - minimal : minimal envelope (no optional fields)
    - standard: standard envelope (representative merchant & amount)
    - max_meta: envelope with previous event hash (32B) and risk class annotations

Measures:
    - payload_bytes          : raw serialized size in bytes
    - parse_time_us          : decode latency (microseconds per decode, averaged over 10,000 runs)
    - signing_input_bytes    : size of domain-separated signing input buffer
    - sms_segments           : number of SMS segments needed when transported via Base64 GSM SMS
    - compression_ratio_vs_json : CBOR size savings relative to minified JSON

Results written to simulator/results/e2_encoding_comparison.json.
Reference: docs/07-research-experiments/EXPERIMENT_PLAN_DETAIL.md E2
"""

from __future__ import annotations

import json
import subprocess
import sys
import time
from pathlib import Path

_SIMULATOR_ROOT = Path(__file__).parent.parent
sys.path.insert(0, str(_SIMULATOR_ROOT))

from resilientpay_sim.domain.crypto import (
    SIGNING_DOMAIN_SEPARATOR,
    encode_envelope_cbor,
)
from resilientpay_sim.domain.model import Money, PaymentEnvelope

NUM_ITERATIONS = 10_000
PROTOCOL_VERSION_STR = "1.0"
SMS_MAX_CHAR_PER_SEGMENT = 136  # Base64 envelope segment ceiling per SMS_PROTOCOL.md


def _git_commit() -> str:
    try:
        return subprocess.check_output(
            ["git", "rev-parse", "HEAD"],
            cwd=str(_SIMULATOR_ROOT),
            stderr=subprocess.DEVNULL,
        ).decode().strip()
    except Exception:
        return "unknown"


def to_compact_json(env: PaymentEnvelope) -> str:
    """Produces canonical compact JSON without whitespace."""
    data = {
        "v": env.protocol_version,
        "tx": str(env.tx_id),
        "cr": str(env.credential_id),
        "pk": str(env.payer_key_id),
        "m": str(env.merchant_id),
        "a": env.amount.amount_minor,
        "c": env.amount.currency,
        "ctr": env.counter,
        "n": env.nonce.hex(),
        "cat": env.created_at_unix,
        "eat": env.expires_at_unix,
    }
    if env.previous_event_hash:
        data["peh"] = env.previous_event_hash.hex()
    if env.risk_class:
        data["rc"] = env.risk_class
    return json.dumps(data, separators=(",", ":"))


def main() -> None:
    print("=" * 65)
    print("E2 — Encoding Comparison (Canonical CBOR vs Compact JSON)")
    print("=" * 65)

    cases = [
        (
            "minimal",
            PaymentEnvelope(
                protocol_version=1,
                tx_id="00000000-0000-0000-0000-000000000001",
                credential_id="00000000-0000-0000-0000-000000000002",
                payer_key_id="00000000-0000-0000-0000-000000000003",
                merchant_id="00000000-0000-0000-0000-000000000004",
                amount=Money(150, "INR"),
                counter=1,
                nonce=b"\x01" * 16,
                created_at_unix=1700000000,
                expires_at_unix=1700003600,
            ),
        ),
        (
            "standard",
            PaymentEnvelope(
                protocol_version=1,
                tx_id="a1b2c3d4-e5f6-4a5b-8c9d-0e1f2a3b4c5d",
                credential_id="b2c3d4e5-f6a1-4b5c-9d0e-1f2a3b4c5d6e",
                payer_key_id="c3d4e5f6-a1b2-4c5d-0e1f-2a3b4c5d6e7f",
                merchant_id="d4e5f6a1-b2c3-4d5e-1f2a-3b4c5d6e7f80",
                amount=Money(50000, "INR"),
                counter=42,
                nonce=b"\xab" * 16,
                created_at_unix=1700000000,
                expires_at_unix=1700003600,
            ),
        ),
        (
            "max_meta",
            PaymentEnvelope(
                protocol_version=1,
                tx_id="a1b2c3d4-e5f6-4a5b-8c9d-0e1f2a3b4c5d",
                credential_id="b2c3d4e5-f6a1-4b5c-9d0e-1f2a3b4c5d6e",
                payer_key_id="c3d4e5f6-a1b2-4c5d-0e1f-2a3b4c5d6e7f",
                merchant_id="d4e5f6a1-b2c3-4d5e-1f2a-3b4c5d6e7f80",
                amount=Money(200000, "INR"),
                counter=100,
                nonce=b"\xff" * 16,
                created_at_unix=1700000000,
                expires_at_unix=1700003600,
                previous_event_hash=b"\x33" * 32,
                risk_class="LOW",
            ),
        ),
    ]

    results = []
    print(f"{'Case':<10} {'Format':<8} {'Payload B':<10} {'SignInput B':<12} {'Parse Time (us)':<16} {'SMS Segs':<8}")
    print("-" * 65)

    for name, env in cases:
        # 1. Canonical CBOR
        cbor_bytes = encode_envelope_cbor(env)
        cbor_sign_input = SIGNING_DOMAIN_SEPARATOR + cbor_bytes
        cbor_len = len(cbor_bytes)
        cbor_sign_len = len(cbor_sign_input)

        # Benchmark CBOR decode time
        import cbor2
        t0 = time.perf_counter()
        for _ in range(NUM_ITERATIONS):
            _ = cbor2.loads(cbor_bytes)
        t1 = time.perf_counter()
        cbor_parse_us = ((t1 - t0) / NUM_ITERATIONS) * 1e6

        # Estimated Base64 SMS segments for (envelope + 64B sig)
        # combined size: cbor_len + 64
        import base64
        cbor_b64 = base64.b64encode(cbor_bytes + b"\x00" * 64).decode()
        cbor_sms_segs = 2 if (cbor_len + 64) <= 170 else (len(cbor_b64) // SMS_MAX_CHAR_PER_SEGMENT + 1)

        # 2. Compact JSON
        json_str = to_compact_json(env)
        json_bytes = json_str.encode("utf-8")
        json_sign_input = SIGNING_DOMAIN_SEPARATOR + json_bytes
        json_len = len(json_bytes)
        json_sign_len = len(json_sign_input)

        # Benchmark JSON decode time
        t0 = time.perf_counter()
        for _ in range(NUM_ITERATIONS):
            _ = json.loads(json_str)
        t1 = time.perf_counter()
        json_parse_us = ((t1 - t0) / NUM_ITERATIONS) * 1e6

        json_b64 = base64.b64encode(json_bytes + b"\x00" * 64).decode()
        json_sms_segs = (len(json_b64) + SMS_MAX_CHAR_PER_SEGMENT - 1) // SMS_MAX_CHAR_PER_SEGMENT

        savings_pct = (1.0 - (cbor_len / json_len)) * 100

        print(f"{name:<10} {'CBOR':<8} {cbor_len:<10} {cbor_sign_len:<12} {cbor_parse_us:<16.2f} {cbor_sms_segs:<8}")
        print(f"{name:<10} {'JSON':<8} {json_len:<10} {json_sign_len:<12} {json_parse_us:<16.2f} {json_sms_segs:<8}")
        print(f"   -> CBOR payload savings: {savings_pct:.1f}%\n")

        results.append({
            "case": name,
            "cbor": {
                "payload_bytes": cbor_len,
                "signing_input_bytes": cbor_sign_len,
                "parse_time_us": round(cbor_parse_us, 2),
                "sms_segments": cbor_sms_segs,
            },
            "compact_json": {
                "payload_bytes": json_len,
                "signing_input_bytes": json_sign_len,
                "parse_time_us": round(json_parse_us, 2),
                "sms_segments": json_sms_segs,
            },
            "cbor_byte_savings_percent": round(savings_pct, 1),
        })

    out_file = _SIMULATOR_ROOT / "results" / "e2_encoding_comparison.json"
    payload = {
        "experiment": "E2_encoding_comparison",
        "schema_version": 1,
        "metadata": {
            "timestamp_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "git_commit": _git_commit(),
            "iterations_benchmarked": NUM_ITERATIONS,
            "protocol_version": PROTOCOL_VERSION_STR,
        },
        "results": results,
    }
    with open(out_file, "w") as f:
        json.dump(payload, f, indent=2)

    print(f"Results written to: {out_file}")


if __name__ == "__main__":
    main()
