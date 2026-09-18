#!/usr/bin/env python3
"""
E4 — Double-Spend Simulation
==============================
Evaluates the protocol's resistance to double-spend attempts under varied
offline parameters.

Parameter sweep:
    offline_budget_paise      : 100 / 500 / 2000
    credential_lifetime_mins  : 5 / 30 / 1440
    attacker_strategy         : replay_same_counter
                                reuse_credential_after_budget_exhausted

50 trials per parameter combination.

Measures:
    - economic_exposure_paise  : maximum undetected value before backend reconciles
                                 (0 in this protocol because replay is detected
                                  at reconciliation time via duplicate counter)
    - conflict_detection_time_txns : how many txns until backend sees a CONFLICT
    - false_acceptance_count   : MUST be 0 for signature-verified paths

Results written to simulator/results/e4_double_spend.json.

Security invariant: false_acceptance_count MUST be 0. Any non-zero value is a
protocol defect and MUST be reported as a stop condition.

Reference: docs/07-research-experiments/EXPERIMENT_PLAN_DETAIL.md E4
           docs/05-protocol/OFFLINE_CREDENTIAL_SPEC.md
           docs/05-protocol/RECONCILIATION_SPEC.md §3
"""

from __future__ import annotations

import json
import subprocess
import sys
import time
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import List

_SIMULATOR_ROOT = Path(__file__).parent.parent
sys.path.insert(0, str(_SIMULATOR_ROOT))

import attr
from resilientpay_sim.domain.crypto import keypair_from_seed, sign_envelope
from resilientpay_sim.domain.model import (
    ConnectivityState,
    Credential,
    CredentialState,
    Money,
    PaymentEnvelope,
    TransactionState,
)
from resilientpay_sim.engine import PROTOCOL_VERSION, ReconciliationEngine

TRIALS_PER_COMBINATION = 50
PROTOCOL_VERSION_STR = "1.0"

OFFLINE_BUDGETS = [100, 500, 2000]           # paise
CREDENTIAL_LIFETIMES_MINS = [5, 30, 1440]   # minutes
ATTACKER_STRATEGIES = [
    "replay_same_counter",
    "reuse_credential_after_budget_exhausted",
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


# ---------------------------------------------------------------------------
# Double-spend trial runner
# ---------------------------------------------------------------------------


def _run_trial(
    seed: int,
    offline_budget_paise: int,
    lifetime_secs: int,
    strategy: str,
) -> dict:
    """
    Simulate a double-spend attempt.

    The attacker has a valid credential and tries to spend more than allowed
    by the offline budget by replaying or reusing credentials.

    Returns per-trial results.
    """
    import random
    rng = random.Random(seed)
    sim_time = 1_700_000_000

    private_seed = bytes(rng.randint(0, 255) for _ in range(32))
    private_seed, public_key = keypair_from_seed(private_seed)

    cred = Credential(
        credential_id=str(uuid.UUID(int=rng.getrandbits(128))),
        subject_key_id=str(uuid.UUID(int=rng.getrandbits(128))),
        public_key_bytes=public_key,
        issued_at_unix=sim_time,
        expires_at_unix=sim_time + lifetime_secs,
        max_value_per_tx_minor=offline_budget_paise,
        max_counter=10,
        state=CredentialState.ACTIVE,
    )

    reconciler = ReconciliationEngine()
    reconciler.register_credential(cred)

    false_acceptance_count = 0
    conflict_detection_txn: int | None = None
    total_txns = 0
    reconciled_value = 0
    max_exposure = 0

    # Strategy: replay_same_counter — attacker submits envelope with counter=1 twice
    if strategy == "replay_same_counter":
        # First submission (legitimate)
        env1 = PaymentEnvelope(
            protocol_version=PROTOCOL_VERSION,
            tx_id=str(uuid.UUID(int=rng.getrandbits(128))),
            credential_id=cred.credential_id,
            payer_key_id=cred.subject_key_id,
            merchant_id=str(uuid.UUID(int=rng.getrandbits(128))),
            amount=Money(amount_minor=offline_budget_paise // 2, currency="INR"),
            counter=1,
            nonce=bytes(rng.randint(0, 255) for _ in range(16)),
            created_at_unix=sim_time,
            expires_at_unix=sim_time + 3600,
        )
        sig1 = sign_envelope(env1, private_seed)
        env1 = attr.evolve(env1, signature_bytes=sig1)

        state1, _ = reconciler.reconcile(env1, sim_time)
        total_txns += 1
        if state1 == TransactionState.RECONCILED:
            reconciled_value += env1.amount.amount_minor

        # Attacker submits same counter with a *different* tx_id (different content)
        env2 = PaymentEnvelope(
            protocol_version=PROTOCOL_VERSION,
            tx_id=str(uuid.UUID(int=rng.getrandbits(128))),  # different tx_id
            credential_id=cred.credential_id,
            payer_key_id=cred.subject_key_id,
            merchant_id=str(uuid.UUID(int=rng.getrandbits(128))),
            amount=Money(amount_minor=offline_budget_paise // 2, currency="INR"),
            counter=1,  # SAME counter — protocol violation
            nonce=bytes(rng.randint(0, 255) for _ in range(16)),
            created_at_unix=sim_time,
            expires_at_unix=sim_time + 3600,
        )
        sig2 = sign_envelope(env2, private_seed)
        env2 = attr.evolve(env2, signature_bytes=sig2)

        state2, _ = reconciler.reconcile(env2, sim_time)
        total_txns += 1
        if state2 == TransactionState.RECONCILED:
            # False acceptance: attacker got a second RECONCILE on same counter
            false_acceptance_count += 1
            reconciled_value += env2.amount.amount_minor
        elif state2 == TransactionState.CONFLICT:
            if conflict_detection_txn is None:
                conflict_detection_txn = total_txns

    # Strategy: reuse_credential_after_budget_exhausted
    # Attacker exhausts the counter budget then tries to use counter beyond max
    elif strategy == "reuse_credential_after_budget_exhausted":
        counter = 0
        for i in range(cred.max_counter + 3):  # exceed max_counter
            counter += 1
            amount = min(offline_budget_paise, 10)  # small amounts
            nonce = bytes(rng.randint(0, 255) for _ in range(16))
            env = PaymentEnvelope(
                protocol_version=PROTOCOL_VERSION,
                tx_id=str(uuid.UUID(int=rng.getrandbits(128))),
                credential_id=cred.credential_id,
                payer_key_id=cred.subject_key_id,
                merchant_id=str(uuid.UUID(int=rng.getrandbits(128))),
                amount=Money(amount_minor=amount, currency="INR"),
                counter=counter,
                nonce=nonce,
                created_at_unix=sim_time,
                expires_at_unix=sim_time + 3600,
            )
            sig = sign_envelope(env, private_seed)
            env = attr.evolve(env, signature_bytes=sig)

            state, _ = reconciler.reconcile(env, sim_time)
            total_txns += 1

            if state == TransactionState.RECONCILED:
                reconciled_value += env.amount.amount_minor
                if counter > cred.max_counter:
                    # Should not happen — if it does, it's a false acceptance
                    false_acceptance_count += 1
            elif state == TransactionState.CONFLICT:
                if conflict_detection_txn is None:
                    conflict_detection_txn = total_txns

    # Economic exposure: value reconciled beyond legitimate first transaction
    # In replay_same_counter, max legitimate is offline_budget_paise // 2.
    # For the extended budget scenario, legitimate = max_counter * 10.
    if strategy == "replay_same_counter":
        legitimate_ceiling = offline_budget_paise // 2
    else:
        legitimate_ceiling = cred.max_counter * 10
    max_exposure = max(0, reconciled_value - legitimate_ceiling)

    return {
        "seed": seed,
        "false_acceptance_count": false_acceptance_count,
        "conflict_detection_txn": conflict_detection_txn,
        "reconciled_value_paise": reconciled_value,
        "economic_exposure_paise": max_exposure,
        "total_txns_submitted": total_txns,
    }


def _aggregate_trials(trials: list[dict]) -> dict:
    n = len(trials)
    total_false_accept = sum(t["false_acceptance_count"] for t in trials)
    exposures = [t["economic_exposure_paise"] for t in trials]
    detection_times = [t["conflict_detection_txn"] for t in trials if t["conflict_detection_txn"] is not None]
    avg_exposure = sum(exposures) / n if n else 0.0
    max_exposure = max(exposures) if exposures else 0
    avg_detection = sum(detection_times) / len(detection_times) if detection_times else None

    return {
        "trials": n,
        "false_acceptance_count": total_false_accept,
        "economic_exposure_paise_avg": round(avg_exposure, 2),
        "economic_exposure_paise_max": max_exposure,
        "conflict_detection_time_txns_avg": round(avg_detection, 2) if avg_detection is not None else None,
        "conflict_detected_in_n_trials": len(detection_times),
        "per_trial": trials,
    }


def main() -> None:
    results = []

    for budget in OFFLINE_BUDGETS:
        for lifetime_mins in CREDENTIAL_LIFETIMES_MINS:
            lifetime_secs = lifetime_mins * 60
            for strategy in ATTACKER_STRATEGIES:
                trials = [
                    _run_trial(seed, budget, lifetime_secs, strategy)
                    for seed in range(TRIALS_PER_COMBINATION)
                ]
                agg = _aggregate_trials(trials)

                # Security invariant check
                if agg["false_acceptance_count"] > 0:
                    print(
                        f"SECURITY VIOLATION: false_acceptance_count={agg['false_acceptance_count']} "
                        f"for budget={budget} lifetime={lifetime_mins}m strategy={strategy}",
                        file=sys.stderr,
                    )

                results.append({
                    "offline_budget_paise": budget,
                    "credential_lifetime_mins": lifetime_mins,
                    "attacker_strategy": strategy,
                    **agg,
                })

    out_dir = _SIMULATOR_ROOT / "results"
    out_dir.mkdir(parents=True, exist_ok=True)
    out_path = out_dir / "e4_double_spend.json"

    total_false = sum(r["false_acceptance_count"] for r in results)
    report = {
        "experiment": "E4_double_spend",
        "schema_version": 1,
        "metadata": {
            "timestamp_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "git_commit": _git_commit(),
            "trials_per_combination": TRIALS_PER_COMBINATION,
            "protocol_version": PROTOCOL_VERSION_STR,
            "offline_budgets_paise": OFFLINE_BUDGETS,
            "credential_lifetimes_mins": CREDENTIAL_LIFETIMES_MINS,
            "attacker_strategies": ATTACKER_STRATEGIES,
            "total_false_acceptance_count": total_false,
            "security_invariant_passed": total_false == 0,
        },
        "results": results,
    }

    with open(out_path, "w") as f:
        json.dump(report, f, indent=2)

    # ---- Summary table ----
    header = (
        f"{'Budget':>8} {'Lifetime':>9} {'Strategy':<38} "
        f"{'FalseAcc':>9} {'MaxExp(p)':>10} {'DetTxns':>8}"
    )
    sep = "=" * len(header)
    print(sep)
    print("E4 Double-Spend (50 trials per combination)")
    print(sep)
    print(header)
    print("-" * len(header))
    for r in results:
        det = r["conflict_detection_time_txns_avg"]
        det_str = f"{det:.1f}" if det is not None else "N/A"
        print(
            f"{r['offline_budget_paise']:>8}"
            f" {r['credential_lifetime_mins']:>8}m"
            f" {r['attacker_strategy']:<38}"
            f" {r['false_acceptance_count']:>9}"
            f" {r['economic_exposure_paise_max']:>10}"
            f" {det_str:>8}"
        )
    print(sep)
    print(f"\nSecurity invariant (false_acceptance_count==0): {'PASS' if total_false == 0 else 'FAIL'}")
    print(f"Results written to: {out_path}")


if __name__ == "__main__":
    main()
