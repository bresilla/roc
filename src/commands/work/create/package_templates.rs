#[allow(dead_code)]
mod cargo;
mod cmake;
/// Package template system for ROS 2 packages
///
/// This module provides a modular template system that supports multiple build systems:
/// - CMake/C++ packages (ament_cmake)
/// - Python packages (ament_python)
/// - Rust packages (ament_cmake_ros with Cargo)
mod common;
mod python;

// Re-export all template functions for backward compatibility
pub use cmake::*;
pub use common::*;
pub use python::*;

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
        )
        .unwrap();

        assert!(xml.contains("<name>test_package</name>"));
        assert!(xml.contains("<description>A test package</description>"));
        assert!(xml.contains("<build_type>ament_cmake</build_type>"));
    }

    #[test]
    fn test_create_cmake_lists() {
        let cmake =
            create_cmake_lists("test_package", Some(&"test_node".to_string()), None).unwrap();

        assert!(cmake.contains("project(test_package)"));
        assert!(cmake.contains("find_package(rclcpp REQUIRED)"));
        assert!(cmake.contains("ament_package()"));
        assert!(cmake.contains("add_executable(test_node"));
    }

    #[test]
    fn test_create_setup_py() {
        let setup = create_setup_py(
            "test_package",
            Some(&"test_node".to_string()),
            "Test User",
            "test@example.com",
            "Test package",
            "Apache-2.0",
        )
        .unwrap();

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
