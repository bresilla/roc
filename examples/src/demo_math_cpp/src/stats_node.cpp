#include <chrono>
#include <memory>
#include <vector>

#include "rclcpp/rclcpp.hpp"
#include "std_msgs/msg/string.hpp"

#include "demo_math_cpp/math_pipeline.hpp"
#include "demo_math_cpp/stream_catalog.hpp"

using namespace std::chrono_literals;

class StatsNode : public rclcpp::Node
{
public:
  StatsNode()
  : Node("stats_node")
  {
    const auto stream_name = declare_parameter<std::string>("stream_name", "burst");
    const auto topic_name = declare_parameter<std::string>("topic_name", "demo/stats");
    const auto window_size = static_cast<std::size_t>(
      declare_parameter<int64_t>("window_size", 5));
    const auto period_ms = declare_parameter<int64_t>("publish_period_ms", 750);

    auto scenario = demo_math_cpp::find_scenario(stream_name);
    samples_ = scenario.values;
    window_ = std::make_unique<demo_math_cpp::SampleWindow>(
      demo_math_cpp::PipelineOptions{stream_name, window_size});

    publisher_ = create_publisher<std_msgs::msg::String>(topic_name, 10);
    timer_ = create_wall_timer(
      std::chrono::milliseconds(period_ms),
      std::bind(&StatsNode::tick, this));

    RCLCPP_INFO(
      get_logger(),
      "Loaded scenario '%s' with %zu samples",
      stream_name.c_str(),
      samples_.size());
  }

private:
  void tick()
  {
    if (samples_.empty()) {
      return;
    }
    if (cursor_ >= samples_.size()) {
      cursor_ = 0;
    }

    window_->push(samples_[cursor_++]);
    auto summary = window_->summarize();

    std_msgs::msg::String message;
    message.data = summary.to_text();
    publisher_->publish(message);
    RCLCPP_INFO(get_logger(), "Published summary: %s", message.data.c_str());
  }

  rclcpp::Publisher<std_msgs::msg::String>::SharedPtr publisher_;
  rclcpp::TimerBase::SharedPtr timer_;
  std::unique_ptr<demo_math_cpp::SampleWindow> window_;
  std::vector<double> samples_;
  std::size_t cursor_ = 0;
};

int main(int argc, char ** argv)
{
  rclcpp::init(argc, argv);
  rclcpp::spin(std::make_shared<StatsNode>());
  rclcpp::shutdown();
  return 0;
}
