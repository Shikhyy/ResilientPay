#!/usr/bin/env python3
import argparse
import json
import sys
from collections import defaultdict


def main():
    parser = argparse.ArgumentParser(description="Analyse ResilientPay batch experiments")
    parser.add_argument("results_file", type=str, help="Path to results JSON file")
    args = parser.parse_args()

    try:
        with open(args.results_file) as f:
            data = json.load(f)
    except Exception as e:
        print(f"Error reading file: {e}")
        sys.exit(1)

    runs = data.get("runs", [])
    if not runs:
        print("No runs found in results.")
        return

    # Group by scenario
    scenarios = defaultdict(list)
    for run in runs:
        scenarios[run["scenario_id"]].append(run)

    print("=" * 88)
    print("ResilientPay Experiment Analysis")
    print(f"Timestamp: {data.get('run_timestamp')} | Runs: {len(runs)}")
    print("=" * 88)
    print(
        f"{'Scenario':<25} | {'Recon %':<8} | {'Conf %':<8} | {'Rej %':<8} | {'Loss %':<8} | {'Txns (avg)':<10}"
    )
    print("-" * 88)

    for scenario_id, runs_list in scenarios.items():
        total_txns = sum(r["total_transactions"] for r in runs_list)
        total_recon = sum(r["total_reconciled"] for r in runs_list)
        total_conflicts = sum(r["total_conflicts"] for r in runs_list)
        total_rejected = sum(r.get("total_rejected", 0) for r in runs_list)
        total_losses = sum(r["total_transport_losses"] for r in runs_list)

        recon_pct = (total_recon / total_txns * 100) if total_txns > 0 else 0.0
        conf_pct = (total_conflicts / total_txns * 100) if total_txns > 0 else 0.0
        rej_pct = (total_rejected / total_txns * 100) if total_txns > 0 else 0.0
        loss_pct = (total_losses / total_txns * 100) if total_txns > 0 else 0.0
        avg_txns = total_txns / len(runs_list)

        print(
            f"{scenario_id:<25} | {recon_pct:>5.1f}%   | {conf_pct:>5.1f}%   | {rej_pct:>5.1f}%   | {loss_pct:>5.1f}%   | {avg_txns:>8.1f}"
        )

    print("=" * 88)


if __name__ == "__main__":
    main()
