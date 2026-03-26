#include <chrono>
#include <memory>
#include <sstream>
#include <vector>

#include "rclcpp/rclcpp.hpp"
#include "std_msgs/msg/string.hpp"

#include "demo_math_cpp/math_pipeline.hpp"
#include "demo_math_cpp/stream_catalog.hpp"

using namespace std::chrono_literals;

class CatalogReportNode : public rclcpp::Node
{
public:
  CatalogReportNode()
  : Node("catalog_report_node")
  {
    publisher_ = create_publisher<std_msgs::msg::String>("demo/catalog", 10);
    timer_ = create_wall_timer(2s, std::bind(&CatalogReportNode::publish_catalog, this));
  }

private:
  void publish_catalog()
  {
    const auto scenarios = demo_math_cpp::built_in_scenarios();
    std::ostringstream stream;
    stream << "catalog";
    for (const auto & scenario : scenarios) {
      const auto summary = demo_math_cpp::summarize_series(scenario.values, scenario.name);
      stream << " [" << summary.to_text() << "]";
    }

    std_msgs::msg::String message;
    message.data = stream.str();
    publisher_->publish(message);
    RCLCPP_INFO(get_logger(), "Catalog summary: %s", message.data.c_str());
  }

  rclcpp::Publisher<std_msgs::msg::String>::SharedPtr publisher_;
  rclcpp::TimerBase::SharedPtr timer_;
};

int main(int argc, char ** argv)
{
  rclcpp::init(argc, argv);
  rclcpp::spin(std::make_shared<CatalogReportNode>());
  rclcpp::shutdown();
  return 0;
}
