/// Common ROS 2 package templates that are language-agnostic
use std::error::Error;

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn create_package_xml(
    package_name: &str,
    package_format: &str,
    description: &str,
    license: &str,
    maintainer_name: &str,
    maintainer_email: &str,
    build_type: &str,
    dependencies: &[&str],
) -> Result<String, Box<dyn Error>> {
    let package_name_escaped = escape_xml(package_name);
    let description_escaped = escape_xml(description);
    let license_escaped = escape_xml(license);
    let maintainer_name_escaped = escape_xml(maintainer_name);
    let maintainer_email_escaped = escape_xml(maintainer_email);

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
        package_format,
        package_name_escaped,
        description_escaped,
        maintainer_email_escaped,
        maintainer_name_escaped,
        license_escaped
    );

    // Add dependencies
    for dep in dependencies {
        xml.push_str(&format!("  <depend>{}</depend>\n", escape_xml(dep)));
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
        "ament_cmake_ros" => {
            xml.push_str("  <buildtool_depend>ament_cmake</buildtool_depend>\n");
            xml.push_str("  <test_depend>ament_lint_auto</test_depend>\n");
            xml.push_str("  <test_depend>ament_lint_common</test_depend>\n");
        }
        _ => {}
    }

    xml.push_str("  <export>\n");
    xml.push_str(&format!("    <build_type>{}</build_type>\n", build_type));
    xml.push_str("  </export>\n");
    xml.push_str("</package>\n");

    Ok(xml)
}

pub fn capitalize_first_letter(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::create_package_xml;

    #[test]
    fn escapes_xml_sensitive_fields() {
        let xml = create_package_xml(
            "demo_pkg",
            "3",
            "Rock & Roll <demo>",
            "Apache-2.0",
            "O'Neil",
            "oneil@example.com",
            "ament_cmake",
            &["std_msgs"],
        )
        .unwrap();

        assert!(xml.contains("Rock &amp; Roll &lt;demo&gt;"));
        assert!(xml.contains("O&apos;Neil"));
    }
}
