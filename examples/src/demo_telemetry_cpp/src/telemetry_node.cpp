#include <chrono>
#include <memory>
#include <vector>

#include "rclcpp/rclcpp.hpp"
#include "std_msgs/msg/string.hpp"

#include "demo_math_cpp/math_pipeline.hpp"
#include "demo_math_cpp/stream_catalog.hpp"
#include "demo_telemetry_cpp/telemetry_formatter.hpp"

using namespace std::chrono_literals;

class TelemetryNode : public rclcpp::Node
{
public:
  TelemetryNode()
  : Node("telemetry_node")
  {
    const auto scenario_name = declare_parameter<std::string>("scenario_name", "descending");
    const auto topic_name = declare_parameter<std::string>("topic_name", "demo/telemetry");
    const auto window_size = static_cast<std::size_t>(
      declare_parameter<int64_t>("window_size", 8));
    const auto period_ms = declare_parameter<int64_t>("publish_period_ms", 1000);

    const auto scenario = demo_math_cpp::find_scenario(scenario_name);
    samples_ = scenario.values;
    rolling_window_ = std::make_unique<demo_math_cpp::SampleWindow>(
      demo_math_cpp::PipelineOptions{scenario_name, window_size});

    publisher_ = create_publisher<std_msgs::msg::String>(topic_name, 10);
    timer_ = create_wall_timer(
      std::chrono::milliseconds(period_ms),
      std::bind(&TelemetryNode::publish_report, this));
  }

private:
  void publish_report()
  {
    if (samples_.empty()) {
      return;
    }
    if (cursor_ >= samples_.size()) {
      cursor_ = 0;
    }

    const auto value = samples_[cursor_++];
    rolling_window_->push(value);
    const auto summary = rolling_window_->summarize();

    std_msgs::msg::String message;
    message.data = demo_telemetry_cpp::render_report("telemetry_node", summary, value);
    publisher_->publish(message);
    RCLCPP_INFO(get_logger(), "Telemetry report: %s", message.data.c_str());
  }

  rclcpp::Publisher<std_msgs::msg::String>::SharedPtr publisher_;
  rclcpp::TimerBase::SharedPtr timer_;
  std::unique_ptr<demo_math_cpp::SampleWindow> rolling_window_;
  std::vector<double> samples_;
  std::size_t cursor_ = 0;
};

int main(int argc, char ** argv)
{
  rclcpp::init(argc, argv);
  rclcpp::spin(std::make_shared<TelemetryNode>());
  rclcpp::shutdown();
  return 0;
}
