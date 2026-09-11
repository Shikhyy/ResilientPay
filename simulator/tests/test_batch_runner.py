import pytest
from resilientpay_sim.scenarios.catalogue import ALL_SCENARIOS
from resilientpay_sim.engine import Simulator

def test_all_scenarios_produce_valid_results():
    for scenario_name, config_factory in ALL_SCENARIOS.items():
        config = config_factory(seed=1)
        sim = Simulator(config)
        result = sim.run()

        assert result.run_id is not None
        assert result.scenario_id.startswith(scenario_name)
        assert result.seed == 1
        assert result.total_transactions > 0
        assert result.total_reconciled >= 0
        assert result.total_rejected >= 0
        assert result.total_conflicts >= 0
        assert result.total_transport_losses >= 0
        assert result.total_transport_duplicates >= 0

