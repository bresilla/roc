from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class Scenario:
    name: str
    description: str
    values: list[float]


def available_scenarios() -> list[Scenario]:
    return [
        Scenario(
            name="steady",
            description="Mostly flat series with tiny drift.",
            values=[1.0, 1.05, 1.0, 0.98, 1.01, 1.0, 1.02],
        ),
        Scenario(
            name="burst",
            description="Short burst in the middle of a quiet baseline.",
            values=[0.8, 0.82, 0.81, 3.5, 4.0, 2.0, 1.0],
        ),
        Scenario(
            name="descending",
            description="Steady decline from a hot starting point.",
            values=[5.0, 4.0, 3.5, 3.0, 2.5, 2.0, 1.5],
        ),
    ]


def load_named_series(name: str) -> list[float]:
    for scenario in available_scenarios():
        if scenario.name == name:
            return list(scenario.values)
    raise KeyError(f"unknown scenario: {name}")
