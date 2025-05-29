// Package template generators for ROS 2 packages
use std::fmt::Write;

/// Creates a package.xml file content
pub fn create_package_xml(
    package_name: &str,
    package_format: &str,
    description: &str,
    license: &str,
    maintainer_name: &str,
    maintainer_email: &str,
    build_type: &str,
    dependencies: &[&str],
) -> Result<String, Box<dyn std::error::Error>> {
    let mut xml = String::new();
    
    writeln!(xml, "<?xml version=\"1.0\"?>")?;
    writeln!(xml, "<?xml-model href=\"http://download.ros.org/schema/package_format{}.xsd\" schematypens=\"http://www.w3.org/2001/XMLSchema\"?>", package_format)?;
    writeln!(xml, "<package format=\"{}\">", package_format)?;
    writeln!(xml, "  <name>{}</name>", package_name)?;
    writeln!(xml, "  <version>0.0.0</version>")?;
    writeln!(xml, "  <description>{}</description>", description)?;
    writeln!(xml, "")?;
    writeln!(xml, "  <maintainer email=\"{}\">{}</maintainer>", maintainer_email, maintainer_name)?;
    writeln!(xml, "")?;
    writeln!(xml, "  <license>{}</license>", license)?;
    writeln!(xml, "")?;

    // Add buildtool dependencies
    match build_type {
        "ament_cmake" | "cmake" => {
            writeln!(xml, "  <buildtool_depend>ament_cmake</buildtool_depend>")?;
        }
        "ament_python" => {
            writeln!(xml, "  <buildtool_depend>ament_python</buildtool_depend>")?;
        }
        _ => {}
    }

    // Add standard dependencies
    if build_type != "cmake" {
        writeln!(xml, "")?;
        if build_type == "ament_cmake" {
            writeln!(xml, "  <depend>rclcpp</depend>")?;
        } else {
            writeln!(xml, "  <depend>rclpy</depend>")?;
        }
    }

    // Add user-specified dependencies
    for dep in dependencies {
        writeln!(xml, "  <depend>{}</depend>", dep)?;
    }

    // Add test dependencies
    writeln!(xml, "")?;
    writeln!(xml, "  <test_depend>ament_lint_auto</test_depend>")?;
    writeln!(xml, "  <test_depend>ament_lint_common</test_depend>")?;

    // Add export section
    writeln!(xml, "")?;
    writeln!(xml, "  <export>")?;
    writeln!(xml, "    <build_type>{}</build_type>", build_type)?;
    writeln!(xml, "  </export>")?;
    writeln!(xml, "</package>")?;

    Ok(xml)
}

/// Creates CMakeLists.txt content for ament_cmake packages
pub fn create_cmake_lists(
    package_name: &str,
    node_name: Option<&String>,
    library_name: Option<&String>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut cmake = String::new();
    
    writeln!(cmake, "cmake_minimum_required(VERSION 3.8)")?;
    writeln!(cmake, "project({})", package_name)?;
    writeln!(cmake, "")?;
    writeln!(cmake, "if(CMAKE_COMPILER_IS_GNUCXX OR CMAKE_CXX_COMPILER_ID MATCHES \"Clang\")")?;
    writeln!(cmake, "  add_compile_options(-Wall -Wextra -Wpedantic)")?;
    writeln!(cmake, "endif()")?;
    writeln!(cmake, "")?;
    writeln!(cmake, "# find dependencies")?;
    writeln!(cmake, "find_package(ament_cmake REQUIRED)")?;
    writeln!(cmake, "find_package(rclcpp REQUIRED)")?;
    writeln!(cmake, "")?;

    // Add library if specified
    if let Some(lib_name) = library_name {
        writeln!(cmake, "# Create library")?;
        writeln!(cmake, "add_library({} src/{}.cpp)", lib_name, lib_name)?;
        writeln!(cmake, "target_include_directories({} PUBLIC", lib_name)?;
        writeln!(cmake, "  $<BUILD_INTERFACE:${{CMAKE_CURRENT_SOURCE_DIR}}/include>")?;
        writeln!(cmake, "  $<INSTALL_INTERFACE:include>)")?;
        writeln!(cmake, "target_compile_features({} PUBLIC c_std_99 cxx_std_17)  # Require C99 and C++17", lib_name)?;
        writeln!(cmake, "ament_target_dependencies({} rclcpp)", lib_name)?;
        writeln!(cmake, "")?;
    }

    // Add executable if specified
    if let Some(exe_name) = node_name {
        writeln!(cmake, "# Create executable")?;
        writeln!(cmake, "add_executable({} src/{}.cpp)", exe_name, exe_name)?;
        writeln!(cmake, "target_include_directories({} PUBLIC", exe_name)?;
        writeln!(cmake, "  $<BUILD_INTERFACE:${{CMAKE_CURRENT_SOURCE_DIR}}/include>")?;
        writeln!(cmake, "  $<INSTALL_INTERFACE:include>)")?;
        writeln!(cmake, "target_compile_features({} PUBLIC c_std_99 cxx_std_17)  # Require C99 and C++17", exe_name)?;
        writeln!(cmake, "ament_target_dependencies({} rclcpp)", exe_name)?;
        writeln!(cmake, "")?;
    }

    // Add install targets
    if library_name.is_some() || node_name.is_some() {
        writeln!(cmake, "# Install targets")?;
        if let Some(lib_name) = library_name {
            writeln!(cmake, "install(TARGETS {}", lib_name)?;
            writeln!(cmake, "  DESTINATION lib/${{PROJECT_NAME}})")?;
        }
        if let Some(exe_name) = node_name {
            writeln!(cmake, "install(TARGETS {}", exe_name)?;
            writeln!(cmake, "  DESTINATION lib/${{PROJECT_NAME}})")?;
        }
        writeln!(cmake, "")?;
    }

    // Add header installation if library is present
    if library_name.is_some() {
        writeln!(cmake, "# Install headers")?;
        writeln!(cmake, "install(DIRECTORY include/")?;
        writeln!(cmake, "  DESTINATION include)")?;
        writeln!(cmake, "")?;
    }

    // Add testing
    writeln!(cmake, "if(BUILD_TESTING)")?;
    writeln!(cmake, "  find_package(ament_lint_auto REQUIRED)")?;
    writeln!(cmake, "  # the following line skips the linter which checks for copyrights")?;
    writeln!(cmake, "  # comment the line when a copyright and license is added to all source files")?;
    writeln!(cmake, "  set(ament_cmake_copyright_FOUND TRUE)")?;
    writeln!(cmake, "  # the following line skips cpplint (only works in a git repo)")?;
    writeln!(cmake, "  # comment the line when this package is in a git repo and when")?;
    writeln!(cmake, "  # a copyright and license is added to all source files")?;
    writeln!(cmake, "  set(ament_cmake_cpplint_FOUND TRUE)")?;
    writeln!(cmake, "  ament_lint_auto_find_test_dependencies()")?;
    writeln!(cmake, "endif()")?;
    writeln!(cmake, "")?;
    writeln!(cmake, "ament_package()")?;

    Ok(cmake)
}

/// Creates setup.py content for ament_python packages
pub fn create_setup_py(package_name: &str, node_name: Option<&String>) -> Result<String, Box<dyn std::error::Error>> {
    let mut setup_content = format!(r#"from setuptools import find_packages, setup

package_name = '{}'

setup(
    name=package_name,
    version='0.0.0',
    packages=find_packages(exclude=['test']),
    data_files=[
        ('share/ament_index/resource_index/packages',
            ['resource/' + package_name]),
        ('share/' + package_name, ['package.xml']),
    ],
    install_requires=['setuptools'],
    zip_safe=True,
    maintainer='todo',
    maintainer_email='todo@todo.todo',
    description='TODO: Package description',
    license='TODO: License declaration',
    tests_require=['pytest'],
    entry_points={{
        'console_scripts': ["#, package_name);

    if let Some(node_name_str) = node_name {
        setup_content.push_str(&format!("\n            '{}={}.{}:main',", node_name_str, package_name, node_name_str));
    }

    setup_content.push_str(r#"
        ],
    }},
)
"#);

    Ok(setup_content)
}

/// Creates setup.cfg content for ament_python packages
pub fn create_setup_cfg() -> String {
    r#"[develop]
script_dir=$base/lib/[PROJECT_NAME]
[install]
install_scripts=$base/lib/[PROJECT_NAME]
"#.to_string()
}

/// Creates resource file content for ament_python packages
pub fn create_resource_file() -> String {
    String::new() // Empty file as per ROS 2 convention
}

/// Creates a basic C++ node template
pub fn create_cpp_node_template(package_name: &str, node_name: &str) -> String {
    let class_name = to_camel_case(node_name);
    format!(r#"#include <chrono>
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
    size_t count_ = 0;
}};

int main(int argc, char * argv[])
{{
  rclcpp::init(argc, argv);
  rclcpp::spin(std::make_shared<{}>());
  rclcpp::shutdown();
  return 0;
}}
"#, class_name, class_name, node_name, class_name, class_name)
}

/// Creates a basic C++ header template
pub fn create_cpp_header_template(package_name: &str, library_name: &str) -> String {
    let class_name = to_camel_case(library_name);
    let header_guard = format!("{}__{}_{}_HPP_", 
        package_name.to_uppercase().replace('-', "_"), 
        library_name.to_uppercase().replace('-', "_"),
        library_name.to_uppercase().replace('-', "_")
    );
    
    format!(r#"#ifndef {}
#define {}

namespace {}
{{

class {}
{{
public:
  {}();
  virtual ~{}();

private:
  // Add your private members here
}};

}}  // namespace {}

#endif  // {}
"#, header_guard, header_guard, package_name, class_name, class_name, class_name, package_name, header_guard)
}

/// Creates a basic C++ source template
pub fn create_cpp_source_template(package_name: &str, library_name: &str) -> String {
    let class_name = to_camel_case(library_name);
    
    format!(r#"#include "{}/{}.hpp"

namespace {}
{{

{}::{}()
{{
  // Constructor implementation
}}

{}::~{}()
{{
  // Destructor implementation
}}

}}  // namespace {}
"#, package_name, library_name, package_name, class_name, class_name, class_name, class_name, package_name)
}

/// Creates a basic Python node template
pub fn create_python_node_template(package_name: &str, node_name: &str) -> String {
    let class_name = to_camel_case(node_name);
    format!(r#"#!/usr/bin/env python3

import rclpy
from rclpy.node import Node

from std_msgs.msg import String


class {}(Node):

    def __init__(self):
        super().__init__('{}')
        self.publisher_ = self.create_publisher(String, 'topic', 10)
        timer_period = 0.5  # seconds
        self.timer = self.create_timer(timer_period, self.timer_callback)
        self.i = 0

    def timer_callback(self):
        msg = String()
        msg.data = 'Hello World: %d' % self.i
        self.publisher_.publish(msg)
        self.get_logger().info('Publishing: "%s"' % msg.data)
        self.i += 1


def main(args=None):
    rclpy.init(args=args)

    {} = {}()

    rclpy.spin({})

    # Destroy the node explicitly
    # (optional - otherwise it will be done automatically
    # when the garbage collector destroys the node object)
    {}.destroy_node()
    rclpy.shutdown()


if __name__ == '__main__':
    main()
"#, class_name, node_name, node_name, class_name, node_name, node_name)
}

/// Creates Python __init__.py file
pub fn create_python_init() -> String {
    String::new() // Empty __init__.py file
}

/// Gets standard test file templates
pub fn get_test_template(template_name: &str) -> String {
    match template_name {
        "test_copyright.py" => {
            r#"# Copyright 2015 Open Source Robotics Foundation, Inc.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

from ament_copyright.main import main
import pytest


@pytest.mark.copyright
@pytest.mark.linter
def test_copyright():
    rc = main(argv=['.', 'test'])
    assert rc == 0
"#.to_string()
        }
        "test_flake8.py" => {
            r#"# Copyright 2017 Open Source Robotics Foundation, Inc.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

from ament_flake8.main import main_with_errors
import pytest


@pytest.mark.flake8
@pytest.mark.linter
def test_flake8():
    rc, errors = main_with_errors(argv=[])
    assert rc == 0, \
        'Found %d code style errors / warnings:\n' % len(errors) + \
        '\n'.join(errors)
"#.to_string()
        }
        "test_pep257.py" => {
            r#"# Copyright 2015 Open Source Robotics Foundation, Inc.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

from ament_pep257.main import main
import pytest


@pytest.mark.linter
@pytest.mark.pep257
def test_pep257():
    rc = main(argv=['.', 'test'])
    assert rc == 0
"#.to_string()
        }
        _ => String::new(),
    }
}

/// Creates Python test file templates (wrapper for get_test_template)
pub fn create_python_test_template(template_name: &str) -> String {
    get_test_template(template_name)
}

/// Converts snake_case to CamelCase
fn to_camel_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
            }
        })
        .collect()
}
