from demo_python_tools.metrics_cli import build_report, demo_series_names, summarize
from demo_python_tools.scenario_loader import load_named_series


def test_demo_series_names_are_available():
    assert {"steady", "burst", "descending"} <= set(demo_series_names())


def test_summarize_builds_expected_report():
    report = summarize("fixture", [1.0, 2.0, 4.0, 5.0])

    assert report.name == "fixture"
    assert report.count == 4
    assert report.minimum == 1.0
    assert report.maximum == 5.0
    assert report.average == 3.0
    assert report.render() == "series=fixture count=4 min=1.00 max=5.00 avg=3.00"


def test_build_report_applies_window():
    report = build_report("burst", window=3)

    assert report.name == "burst"
    assert report.count == 3


def test_load_named_series_returns_copy():
    values = load_named_series("steady")
    values.append(42.0)

    assert 42.0 not in load_named_series("steady")
