#!/usr/bin/env python3
import argparse
import json
import time
import sys
from pathlib import Path
import dataclasses

# Add the parent directory to sys.path so we can import resilientpay_sim
sys.path.insert(0, str(Path(__file__).parent.parent))

from resilientpay_sim.scenarios.catalogue import ALL_SCENARIOS
from resilientpay_sim.engine import Simulator

def main():
    parser = argparse.ArgumentParser(description="Run ResilientPay batch experiments")
    parser.add_argument("--seeds", type=int, default=10, help="Number of seeds to run (1..N)")
    default_out = f"results/experiment_{int(time.time())}.json"
    parser.add_argument("--output", type=str, default=default_out, help="Output JSON file path")
    args = parser.parse_args()

    out_path = Path(__file__).parent.parent / args.output
    out_path.parent.mkdir(parents=True, exist_ok=True)

    results_list = []
    
    print(f"Running experiments (seeds: 1..{args.seeds}) across {len(ALL_SCENARIOS)} scenarios...")
    print("-" * 80)
    print(f"{'Scenario':<25} | {'Seed':<4} | {'Txns':<5} | {'Reconciled':<10} | {'Rejected':<8} | {'Conflicts':<9}")
    print("-" * 80)

    for scenario_name, config_factory in ALL_SCENARIOS.items():
        for seed in range(1, args.seeds + 1):
            config = config_factory(seed)
            sim = Simulator(config)
            result = sim.run()

            res_dict = {
                "run_id": result.run_id,
                "scenario_id": result.scenario_id,
                "seed": result.seed,
                "total_transactions": result.total_transactions,
                "total_reconciled": result.total_reconciled,
                "total_rejected": result.total_rejected,
                "total_conflicts": result.total_conflicts,
                "total_transport_losses": result.total_transport_losses,
                "total_transport_duplicates": result.total_transport_duplicates,
            }
            results_list.append(res_dict)

            print(f"{scenario_name:<25} | {seed:<4} | {result.total_transactions:<5} | {result.total_reconciled:<10} | {result.total_rejected:<8} | {result.total_conflicts:<9}")

    print("-" * 80)

    final_report = {
        "schema_version": 1,
        "run_timestamp": int(time.time()),
        "protocol_version": 1,
        "generator": "run_experiments.py",
        "runs": results_list,
    }

    with open(out_path, "w") as f:
        json.dump(final_report, f, indent=2)
    
    print(f"Wrote results to {out_path}")

if __name__ == "__main__":
    main()
