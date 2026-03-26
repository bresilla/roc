from __future__ import annotations

import argparse
from dataclasses import dataclass
from statistics import mean
from typing import Iterable

from demo_python_tools.scenario_loader import available_scenarios, load_named_series


@dataclass
class MetricsReport:
    name: str
    count: int
    minimum: float
    maximum: float
    average: float

    def render(self) -> str:
        return (
            f"series={self.name} "
            f"count={self.count} "
            f"min={self.minimum:.2f} "
            f"max={self.maximum:.2f} "
            f"avg={self.average:.2f}"
        )


def summarize(name: str, values: Iterable[float]) -> MetricsReport:
    series = list(values)
    if not series:
        raise ValueError("summarize() requires at least one value")

    return MetricsReport(
        name=name,
        count=len(series),
        minimum=min(series),
        maximum=max(series),
        average=mean(series),
    )


def build_report(name: str, window: int | None = None) -> MetricsReport:
    series = load_named_series(name)
    if window is not None:
        if window <= 0:
            raise ValueError("window must be positive")
        series = series[:window]
    return summarize(name, series)


def demo_series_names() -> list[str]:
    return [scenario.name for scenario in available_scenarios()]


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Render example analytics reports.")
    parser.add_argument(
        "--series",
        default="steady",
        choices=demo_series_names(),
        help="Named input scenario to summarize.",
    )
    parser.add_argument(
        "--window",
        type=int,
        default=None,
        help="Optional prefix window size to summarize.",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> None:
    args = parse_args(argv)
    report = build_report(args.series, args.window)
    print(report.render())
