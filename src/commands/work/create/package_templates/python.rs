use super::common::capitalize_first_letter;
/// Python specific ROS 2 package templates
use std::error::Error;

fn escape_python_single_quoted(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
        .replace('\'', "\\'")
}

pub fn create_setup_py(
    package_name: &str,
    node_name: Option<&String>,
    maintainer_name: &str,
    maintainer_email: &str,
    description: &str,
    license: &str,
) -> Result<String, Box<dyn Error>> {
    let mut entry_points = String::new();
    let maintainer_name = escape_python_single_quoted(maintainer_name);
    let maintainer_email = escape_python_single_quoted(maintainer_email);
    let description = escape_python_single_quoted(description);
    let license = escape_python_single_quoted(license);

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
    maintainer='{}',
    maintainer_email='{}',
    description='{}',
    license='{}',
    tests_require=['pytest'],
    entry_points={{
        'console_scripts': [
{}
        ],
    }},
)
"#,
        package_name, maintainer_name, maintainer_email, description, license, entry_points
    ))
}

pub fn create_setup_cfg(package_name: &str) -> String {
    format!(
        "[develop]\nscript_dir=$base/lib/{}\n[install]\ninstall_scripts=$base/lib/{}\n",
        package_name, package_name
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
        "test_copyright.py" => r#"# Copyright 2015 Open Source Robotics Foundation, Inc.
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
"#
        .to_string(),
        "test_flake8.py" => r#"# Copyright 2017 Open Source Robotics Foundation, Inc.
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
"#
        .to_string(),
        "test_pep257.py" => r#"# Copyright 2015 Open Source Robotics Foundation, Inc.
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
"#
        .to_string(),
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

#[cfg(test)]
mod tests {
    use super::create_setup_py;

    #[test]
    fn escapes_single_quotes_in_setup_metadata() {
        let setup_py = create_setup_py(
            "demo_pkg",
            None,
            "O'Neil",
            "oneil@example.com",
            "robot's package",
            "Apache-2.0",
        )
        .unwrap();

        assert!(setup_py.contains("maintainer='O\\'Neil'"));
        assert!(setup_py.contains("description='robot\\'s package'"));
    }
}
