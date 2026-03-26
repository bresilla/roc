#include "demo_math_cpp/stream_catalog.hpp"

#include <numeric>
#include <stdexcept>

namespace demo_math_cpp
{

std::vector<StreamScenario> built_in_scenarios()
{
  return {
    {
      "steady",
      "Mostly flat signal with small drift.",
      {1.0, 1.1, 1.0, 0.95, 1.05, 1.0, 1.02, 0.99, 1.01, 1.0}
    },
    {
      "burst",
      "Stable baseline followed by a short burst.",
      {0.8, 0.85, 0.82, 0.81, 3.2, 4.1, 3.7, 1.5, 1.0, 0.9}
    },
    {
      "descending",
      "High values rolling down toward idle.",
      {5.0, 4.5, 4.1, 3.9, 3.0, 2.6, 2.2, 1.9, 1.5, 1.2}
    }
  };
}

StreamScenario find_scenario(const std::string & name)
{
  for (const auto & scenario : built_in_scenarios()) {
    if (scenario.name == name) {
      return scenario;
    }
  }
  throw std::runtime_error("unknown stream scenario: " + name);
}

std::map<std::string, double> scenario_snapshot(
  const std::vector<StreamScenario> & scenarios)
{
  std::map<std::string, double> snapshot;
  for (const auto & scenario : scenarios) {
    if (scenario.values.empty()) {
      snapshot.emplace(scenario.name, 0.0);
      continue;
    }

    const auto total = std::accumulate(
      scenario.values.begin(),
      scenario.values.end(),
      0.0);
    snapshot.emplace(
      scenario.name,
      total / static_cast<double>(scenario.values.size()));
  }
  return snapshot;
}

}  // namespace demo_math_cpp
