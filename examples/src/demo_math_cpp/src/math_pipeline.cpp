#include "demo_math_cpp/math_pipeline.hpp"

#include <algorithm>
#include <numeric>
#include <sstream>

namespace demo_math_cpp
{

namespace
{

MessageSummary summarize_values(
  const std::vector<double> & values,
  const std::string & stream_name,
  std::size_t total_samples)
{
  MessageSummary summary;
  if (values.empty()) {
    summary.stream_name = stream_name;
    summary.total_samples = total_samples;
    return summary;
  }

  summary.latest = values.back();
  summary.minimum = *std::min_element(values.begin(), values.end());
  summary.maximum = *std::max_element(values.begin(), values.end());
  summary.spread = summary.maximum - summary.minimum;
  summary.window_size = values.size();
  summary.total_samples = total_samples;
  summary.stream_name = stream_name;
  summary.average = std::accumulate(values.begin(), values.end(), 0.0) /
    static_cast<double>(values.size());
  return summary;
}

}  // namespace

std::string MessageSummary::to_text() const
{
  std::ostringstream stream;
  stream << "stream=" << stream_name
         << " latest=" << latest
         << " avg=" << average
         << " min=" << minimum
         << " max=" << maximum
         << " spread=" << spread
         << " window=" << window_size
         << " total=" << total_samples;
  return stream.str();
}

SampleWindow::SampleWindow(PipelineOptions options)
: options_(std::move(options))
{
}

void SampleWindow::push(double value)
{
  ++total_samples_;
  samples_.push_back(value);
  while (samples_.size() > options_.window_size) {
    samples_.pop_front();
  }
}

MessageSummary SampleWindow::summarize() const
{
  return summarize_values(
    std::vector<double>(samples_.begin(), samples_.end()),
    options_.stream_name,
    total_samples_);
}

bool SampleWindow::empty() const
{
  return samples_.empty();
}

std::size_t SampleWindow::total_samples() const
{
  return total_samples_;
}

const PipelineOptions & SampleWindow::options() const
{
  return options_;
}

MessageSummary summarize_series(
  const std::vector<double> & values,
  const std::string & stream_name)
{
  return summarize_values(values, stream_name, values.size());
}

}  // namespace demo_math_cpp
