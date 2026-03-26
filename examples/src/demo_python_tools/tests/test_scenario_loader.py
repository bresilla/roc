import pytest

from demo_python_tools.metrics_cli import build_report
from demo_python_tools.scenario_loader import available_scenarios, load_named_series


def test_available_scenarios_have_descriptions():
    scenarios = available_scenarios()

    assert len(scenarios) >= 3
    assert all(scenario.description for scenario in scenarios)


def test_build_report_rejects_invalid_window():
    with pytest.raises(ValueError):
        build_report("steady", window=0)


def test_load_named_series_rejects_unknown_name():
    with pytest.raises(KeyError):
        load_named_series("does-not-exist")
