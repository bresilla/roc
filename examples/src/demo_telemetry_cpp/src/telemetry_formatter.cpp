#include "demo_telemetry_cpp/telemetry_formatter.hpp"

#include <sstream>

namespace demo_telemetry_cpp
{

std::string render_report(
  const std::string & source_name,
  const demo_math_cpp::MessageSummary & summary,
  double current_value)
{
  std::ostringstream stream;
  stream << "source=" << source_name
         << " current=" << current_value
         << " " << summary.to_text();
  return stream.str();
}

std::string render_health_line(
  const std::string & latest_stats_message,
  std::size_t received_count)
{
  std::ostringstream stream;
  stream << "monitor_count=" << received_count
         << " latest_stats={" << latest_stats_message << "}";
  return stream.str();
}

}  // namespace demo_telemetry_cpp
