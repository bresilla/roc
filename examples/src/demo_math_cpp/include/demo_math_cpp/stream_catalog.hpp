#ifndef DEMO_MATH_CPP__STREAM_CATALOG_HPP_
#define DEMO_MATH_CPP__STREAM_CATALOG_HPP_

#include <map>
#include <string>
#include <vector>

namespace demo_math_cpp
{

struct StreamScenario
{
  std::string name;
  std::string description;
  std::vector<double> values;
};

std::vector<StreamScenario> built_in_scenarios();
StreamScenario find_scenario(const std::string & name);
std::map<std::string, double> scenario_snapshot(
  const std::vector<StreamScenario> & scenarios);

}  // namespace demo_math_cpp

#endif  // DEMO_MATH_CPP__STREAM_CATALOG_HPP_
