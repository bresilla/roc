
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
    let mut xml = format!(
        r#"<?xml version="1.0"?>
<?xml-model href="http://download.ros.org/schema/package_format3.xsd" schematypens="http://www.w3.org/2001/XMLSchema"?>
<package format="{}">
  <name>{}</name>
  <version>0.0.0</version>
  <description>{}</description>
  <maintainer email="{}">{}</maintainer>
  <license>{}</license>
"#,
        package_format, package_name, description, maintainer_email, maintainer_name, license
    );

    // Add dependencies
    for dep in dependencies {
        xml.push_str(&format!("  <depend>{}</depend>\n", dep));
    }

    // Add build type specific tags
    match build_type {
        "ament_cmake" => {
            xml.push_str("  <buildtool_depend>ament_cmake</buildtool_depend>\n");
            xml.push_str("  <test_depend>ament_lint_auto</test_depend>\n");
            xml.push_str("  <test_depend>ament_lint_common</test_depend>\n");
        }
        "ament_python" => {
            xml.push_str("  <test_depend>ament_flake8</test_depend>\n");
            xml.push_str("  <test_depend>ament_pep257</test_depend>\n");
            xml.push_str("  <test_depend>python3-pytest</test_depend>\n");
        }
        _ => {}
    }

    xml.push_str("  <export>\n");
    xml.push_str(&format!("    <build_type>{}</build_type>\n", build_type));
    xml.push_str("  </export>\n");
    xml.push_str("</package>\n");

    Ok(xml)
}

pub fn create_cmake_lists(
    package_name: &str,
    node_name: Option<&String>,
    library_name: Option<&String>,
) -> Result<String, Box<dyn std::error::Error>> {
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

pub fn create_setup_py(package_name: &str, node_name: Option<&String>) -> Result<String, Box<dyn std::error::Error>> {
    let mut entry_points = String::new();
    
    if let Some(node_name_str) = node_name {
        entry_points = format!(
            "            '{} = {}.{}:main',",
            node_name_str, package_name, node_name_str
        );
    }

    Ok(format!(
        r#"from setuptools import find_packages, setup

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
    maintainer='TODO',
    maintainer_email='todo@example.com',
    description='TODO: Package description',
    license='Apache-2.0',
    tests_require=['pytest'],
    entry_points={{
        'console_scripts': [
{}
        ],
    }},
)
"#,
        package_name, entry_points
    ))
}

pub fn create_setup_cfg() -> String {
    "[develop]\nscript_dir=$base/lib/PACKAGE_NAME\n[install]\ninstall_scripts=$base/lib/PACKAGE_NAME\n".to_string()
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
    let include_guard = format!("{}__{}__HPP_", package_name.to_uppercase(), class_name.to_uppercase());
    
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

pub fn create_python_node_template(_package_name: &str, node_name: &str) -> String {
    format!(
        r#"#!/usr/bin/env python3

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
"#,
        capitalize_first_letter(node_name),
        node_name,
        node_name,
        capitalize_first_letter(node_name),
        node_name,
        node_name
    )
}

pub fn create_python_test_template(test_file: &str) -> String {
    match test_file {
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


# Remove the `skip` decorator once the source file(s) have a copyright header
@pytest.mark.skip(reason='No copyright header has been placed in the generated source file.')
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
        _ => {
            format!(
                r#"import pytest


def test_{}():
    """Test functionality."""
    pass  # Add your tests here
"#,
                test_file.replace(".py", "").replace("test_", "")
            )
        }
    }
}

fn capitalize_first_letter(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_package_xml() {
        let xml = create_package_xml(
            "test_package",
            "3",
            "A test package",
            "Apache-2.0",
            "Test User",
            "test@example.com",
            "ament_cmake",
            &["rclcpp", "std_msgs"],
        ).unwrap();
        
        assert!(xml.contains("<name>test_package</name>"));
        assert!(xml.contains("<description>A test package</description>"));
        assert!(xml.contains("<build_type>ament_cmake</build_type>"));
    }

    #[test]
    fn test_create_cmake_lists() {
        let cmake = create_cmake_lists("test_package", Some(&"test_node".to_string()), None).unwrap();
        
        assert!(cmake.contains("project(test_package)"));
        assert!(cmake.contains("find_package(rclcpp REQUIRED)"));
        assert!(cmake.contains("ament_package()"));
        assert!(cmake.contains("add_executable(test_node"));
    }

    #[test]
    fn test_create_setup_py() {
        let setup = create_setup_py("test_package", Some(&"test_node".to_string())).unwrap();
        
        assert!(setup.contains("name=package_name"));
        assert!(setup.contains("test_node = test_package.test_node:main"));
    }

    #[test]
    fn test_capitalize_first_letter() {
        assert_eq!(capitalize_first_letter("hello"), "Hello");
        assert_eq!(capitalize_first_letter(""), "");
        assert_eq!(capitalize_first_letter("a"), "A");
        assert_eq!(capitalize_first_letter("HELLO"), "HELLO");
    }
}
