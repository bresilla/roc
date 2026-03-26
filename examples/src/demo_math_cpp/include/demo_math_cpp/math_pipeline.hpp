#ifndef DEMO_MATH_CPP__MATH_PIPELINE_HPP_
#define DEMO_MATH_CPP__MATH_PIPELINE_HPP_

#include <cstddef>
#include <deque>
#include <string>
#include <vector>

namespace demo_math_cpp
{

struct MessageSummary
{
  double latest = 0.0;
  double average = 0.0;
  double minimum = 0.0;
  double maximum = 0.0;
  double spread = 0.0;
  std::size_t window_size = 0;
  std::size_t total_samples = 0;
  std::string stream_name;

  std::string to_text() const;
};

struct PipelineOptions
{
  std::string stream_name = "steady";
  std::size_t window_size = 5;
};

class SampleWindow
{
public:
  explicit SampleWindow(PipelineOptions options);

  void push(double value);
  MessageSummary summarize() const;
  bool empty() const;
  std::size_t total_samples() const;
  const PipelineOptions & options() const;

private:
  PipelineOptions options_;
  std::deque<double> samples_;
  std::size_t total_samples_ = 0;
};

MessageSummary summarize_series(
  const std::vector<double> & values,
  const std::string & stream_name);

}  // namespace demo_math_cpp

#endif  // DEMO_MATH_CPP__MATH_PIPELINE_HPP_
