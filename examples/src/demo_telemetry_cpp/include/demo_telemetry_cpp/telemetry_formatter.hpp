#ifndef DEMO_TELEMETRY_CPP__TELEMETRY_FORMATTER_HPP_
#define DEMO_TELEMETRY_CPP__TELEMETRY_FORMATTER_HPP_

#include <string>

#include "demo_math_cpp/math_pipeline.hpp"

namespace demo_telemetry_cpp
{

std::string render_report(
  const std::string & source_name,
  const demo_math_cpp::MessageSummary & summary,
  double current_value);

std::string render_health_line(
  const std::string & latest_stats_message,
  std::size_t received_count);

}  // namespace demo_telemetry_cpp

#endif  // DEMO_TELEMETRY_CPP__TELEMETRY_FORMATTER_HPP_
