#include <memory>
#include <string>

#include "rclcpp/rclcpp.hpp"
#include "std_msgs/msg/string.hpp"

#include "demo_telemetry_cpp/telemetry_formatter.hpp"

class TelemetryMonitorNode : public rclcpp::Node
{
public:
  TelemetryMonitorNode()
  : Node("telemetry_monitor_node")
  {
    const auto input_topic = declare_parameter<std::string>("input_topic", "demo/stats");
    subscription_ = create_subscription<std_msgs::msg::String>(
      input_topic,
      10,
      std::bind(&TelemetryMonitorNode::handle_stats, this, std::placeholders::_1));
  }

private:
  void handle_stats(const std_msgs::msg::String & message)
  {
    ++received_count_;
    latest_stats_message_ = message.data;
    const auto health = demo_telemetry_cpp::render_health_line(
      latest_stats_message_,
      received_count_);
    RCLCPP_INFO(get_logger(), "%s", health.c_str());
  }

  rclcpp::Subscription<std_msgs::msg::String>::SharedPtr subscription_;
  std::size_t received_count_ = 0;
  std::string latest_stats_message_;
};

int main(int argc, char ** argv)
{
  rclcpp::init(argc, argv);
  rclcpp::spin(std::make_shared<TelemetryMonitorNode>());
  rclcpp::shutdown();
  return 0;
}
