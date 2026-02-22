use super::common::capitalize_first_letter;
/// CMake/C++ specific ROS 2 package templates
use std::error::Error;

pub fn create_cmake_lists(
    package_name: &str,
    node_name: Option<&String>,
    library_name: Option<&String>,
) -> Result<String, Box<dyn Error>> {
    let mut cmake = format!(
        r#"cmake_minimum_required(VERSION 3.8)
project({})

if(CMAKE_COMPILER_IS_GNUCXX OR CMAKE_CXX_COMPILER_ID MATCHES "Clang")
  add_compile_options(-Wall -Wextra -Wpedantic)
endif()

# find dependencies
find_package(ament_cmake REQUIRED)
"#,
        package_name
    );

    // Add common ROS 2 dependencies
    cmake.push_str("find_package(rclcpp REQUIRED)\n");
    cmake.push_str("find_package(std_msgs REQUIRED)\n");

    // Add executable targets if node or library is specified
    if let Some(node_name) = node_name {
        cmake.push_str(&format!(
            "\n# Add executable for node\nadd_executable({} src/{}.cpp)\n",
            node_name, node_name
        ));
        // Add include directories for the node if there are headers
        if library_name.is_some() {
            cmake.push_str(&format!(
                "target_include_directories({}\n  PRIVATE\n    ${{CMAKE_CURRENT_SOURCE_DIR}}/include)\n",
                node_name
            ));
        }
        cmake.push_str(&format!(
            "ament_target_dependencies({} rclcpp std_msgs)\n",
            node_name
        ));
        cmake.push_str(&format!(
            "\ninstall(TARGETS {}\n  DESTINATION lib/${{PROJECT_NAME}})\n",
            node_name
        ));
    }

    if let Some(library_name) = library_name {
        cmake.push_str(&format!(
            "\n# Add library\nadd_library({} src/{}.cpp)\ntarget_include_directories({}\n  PUBLIC\n    $<BUILD_INTERFACE:${{CMAKE_CURRENT_SOURCE_DIR}}/include>\n    $<INSTALL_INTERFACE:include>)\nament_target_dependencies({} rclcpp std_msgs)\n",
            library_name, library_name, library_name, library_name
        ));
        cmake.push_str(&format!(
            "\n# Export targets\nament_export_targets({}_targets HAS_LIBRARY_TARGET)\nament_export_dependencies(rclcpp std_msgs)\n",
            library_name
        ));
        cmake.push_str(&format!(
            "\ninstall(TARGETS {}\n  EXPORT {}_targets\n  LIBRARY DESTINATION lib\n  ARCHIVE DESTINATION lib\n  RUNTIME DESTINATION bin)\n",
            library_name, library_name
        ));
        cmake.push_str(&format!(
            "\ninstall(DIRECTORY include/\n  DESTINATION include)\n"
        ));
    }

    cmake.push_str(&format!(
        r#"
if(BUILD_TESTING)
  find_package(ament_lint_auto REQUIRED)
  # the following line skips the linter which checks for copyrights
  # comment the line when a copyright and license is added to all source files
  set(ament_cmake_copyright_FOUND TRUE)
  # the following line skips cpplint (only works in a git repo)
  # comment the line when this package is in a git repo and when
  # a copyright and license is added to all source files
  set(ament_cmake_cpplint_FOUND TRUE)
  ament_lint_auto_find_test_dependencies()
endif()

ament_package()
"#
    ));

    Ok(cmake)
}

pub fn create_cpp_node_template(_package_name: &str, node_name: &str) -> String {
    format!(
        r#"#include <chrono>
#include <functional>
#include <memory>
#include <string>

#include "rclcpp/rclcpp.hpp"
#include "std_msgs/msg/string.hpp"

using namespace std::chrono_literals;

class {} : public rclcpp::Node
{{
public:
  {}()
  : Node("{}")
  {{
    publisher_ = this->create_publisher<std_msgs::msg::String>("topic", 10);
    timer_ = this->create_wall_timer(
      500ms, std::bind(&{}::timer_callback, this));
  }}

private:
  void timer_callback()
  {{
    auto message = std_msgs::msg::String();
    message.data = "Hello, world! " + std::to_string(count_++);
    RCLCPP_INFO(this->get_logger(), "Publishing: '%s'", message.data.c_str());
    publisher_->publish(message);
  }}
  rclcpp::TimerBase::SharedPtr timer_;
  rclcpp::Publisher<std_msgs::msg::String>::SharedPtr publisher_;
  size_t count_;
}};

int main(int argc, char * argv[])
{{
  rclcpp::init(argc, argv);
  rclcpp::spin(std::make_shared<{}>());
  rclcpp::shutdown();
  return 0;
}}
"#,
        capitalize_first_letter(node_name),
        capitalize_first_letter(node_name),
        node_name,
        capitalize_first_letter(node_name),
        capitalize_first_letter(node_name)
    )
}

pub fn create_cpp_header_template(package_name: &str, class_name: &str) -> String {
    let include_guard = format!(
        "{}__{}__HPP_",
        package_name.to_uppercase(),
        class_name.to_uppercase()
    );

    format!(
        r#"#ifndef {}
#define {}

#include <string>

namespace {}
{{

class {}
{{
public:
  {}();
  virtual ~{}();

  void do_something();

private:
  std::string name_;
}};

}}  // namespace {}

#endif  // {}
"#,
        include_guard,
        include_guard,
        package_name,
        class_name,
        class_name,
        class_name,
        package_name,
        include_guard
    )
}

pub fn create_cpp_source_template(package_name: &str, class_name: &str) -> String {
    format!(
        r#"#include "{}/{}.hpp"

#include <iostream>

namespace {}
{{

{}::{}()
: name_("default")
{{
  // Constructor implementation
}}

{}::~{}()
{{
  // Destructor implementation
}}

void {}::do_something()
{{
  std::cout << "Doing something in " << name_ << std::endl;
}}

}}  // namespace {}
"#,
        package_name,
        class_name,
        package_name,
        class_name,
        class_name,
        class_name,
        class_name,
        class_name,
        package_name
    )
}
