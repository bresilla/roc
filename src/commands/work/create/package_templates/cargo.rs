use super::common::capitalize_first_letter;
/// Cargo/Rust specific ROS 2 package templates
use std::error::Error;

#[allow(dead_code)]
pub fn create_cargo_toml(
    package_name: &str,
    node_name: Option<&String>,
    library_name: Option<&String>,
) -> Result<String, Box<dyn Error>> {
    let mut cargo = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
rclrs = "0.4"
std_msgs = "0.3"
tokio = {{ version = "1", features = ["full"] }}
"#,
        package_name
    );

    // Add binary targets for nodes
    if let Some(node_name) = node_name {
        cargo.push_str(&format!(
            r#"
[[bin]]
name = "{}"
path = "src/{}.rs"
"#,
            node_name, node_name
        ));
    }

    // Add library target if specified
    if library_name.is_some() {
        cargo.push_str(
            r#"
[lib]
name = "lib"
path = "src/lib.rs"
"#,
        );
    }

    Ok(cargo)
}

#[allow(dead_code)]
pub fn create_rust_node_template(_package_name: &str, node_name: &str) -> String {
    format!(
        r#"use rclrs::{{Context, Node, Publisher, RclrsError}};
use std_msgs::msg::String as StringMsg;
use std::time::Duration;

struct {}Node {{
    node: Node,
    publisher: Publisher<StringMsg>,
    count: i32,
}}

impl {}Node {{
    fn new(context: &Context) -> Result<Self, RclrsError> {{
        let node = context.create_node("{}")?;
        let publisher = node.create_publisher("topic", rclrs::QOS_PROFILE_DEFAULT)?;
        Ok(Self {{
            node,
            publisher,
            count: 0,
        }})
    }}

    fn timer_callback(&mut self) -> Result<(), RclrsError> {{
        let mut msg = StringMsg::default();
        msg.data = format!("Hello, world! {{}}", self.count);
        println!("Publishing: '{{}}'", msg.data);
        self.publisher.publish(&msg)?;
        self.count += 1;
        Ok(())
    }}
}}

#[tokio::main]
async fn main() -> Result<(), RclrsError> {{
    let context = Context::new(std::env::args())?;
    let mut node = {}Node::new(&context)?;
    
    let mut interval = tokio::time::interval(Duration::from_millis(500));
    
    loop {{
        tokio::select! {{
            _ = interval.tick() => {{
                if let Err(e) = node.timer_callback() {{
                    eprintln!("Error in timer callback: {{}}", e);
                    break;
                }}
            }}
            _ = tokio::signal::ctrl_c() => {{
                println!("Shutting down...");
                break;
            }}
        }}
    }}
    
    Ok(())
}}
"#,
        capitalize_first_letter(node_name),
        capitalize_first_letter(node_name),
        node_name,
        capitalize_first_letter(node_name)
    )
}

#[allow(dead_code)]
pub fn create_rust_lib_template(package_name: &str, class_name: &str) -> String {
    let class_name_cap = capitalize_first_letter(class_name);
    format!(
        r#"//! {} library
//! 
//! This library provides functionality for the {} ROS 2 package.

use rclrs::RclrsError;
use std::fmt;

/// Main library struct for {}
pub struct {} {{
    name: String,
}}

impl {} {{
    /// Create a new instance
    pub fn new() -> Self {{
        Self {{
            name: "default".to_string(),
        }}
    }}

    /// Create a new instance with a custom name
    pub fn with_name(name: String) -> Self {{
        Self {{ name }}
    }}

    /// Do something useful
    pub fn do_something(&self) -> Result<(), {}Error> {{
        println!("Doing something in {{}}", self.name);
        Ok(())
    }}

    /// Get the name
    pub fn name(&self) -> &str {{
        &self.name
    }}
}}

impl Default for {} {{
    fn default() -> Self {{
        Self::new()
    }}
}}

/// Custom error type for {}
#[derive(Debug)]
pub enum {}Error {{
    /// Generic error with message
    Generic(String),
    /// ROS 2 error
    Rclrs(RclrsError),
}}

impl fmt::Display for {}Error {{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {{
        match self {{
            {}Error::Generic(msg) => write!(f, "Generic error: {{}}", msg),
            {}Error::Rclrs(e) => write!(f, "ROS 2 error: {{}}", e),
        }}
    }}
}}

impl std::error::Error for {}Error {{}}

impl From<RclrsError> for {}Error {{
    fn from(error: RclrsError) -> Self {{
        {}Error::Rclrs(error)
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_new() {{
        let lib = {}::new();
        assert_eq!(lib.name(), "default");
    }}

    #[test]
    fn test_with_name() {{
        let lib = {}::with_name("test".to_string());
        assert_eq!(lib.name(), "test");
    }}

    #[test]
    fn test_do_something() {{
        let lib = {}::new();
        assert!(lib.do_something().is_ok());
    }}
}}
"#,
        package_name,   // //! {} library
        package_name,   // //! This library provides functionality for the {} ROS 2 package.
        package_name,   // /// Main library struct for {}
        class_name_cap, // pub struct {} {
        class_name_cap, // impl {} {
        class_name_cap, // pub fn do_something(&self) -> Result<(), {}Error> {
        class_name_cap, // impl Default for {} {
        package_name,   // /// Custom error type for {}
        class_name_cap, // pub enum {}Error {
        class_name_cap, // impl fmt::Display for {}Error {
        class_name_cap, // {}Error::Generic(msg) => write!(f, "Generic error: {{}}", msg),
        class_name_cap, // {}Error::Rclrs(e) => write!(f, "ROS 2 error: {{}}", e),
        class_name_cap, // impl std::error::Error for {}Error {}
        class_name_cap, // impl From<RclrsError> for {}Error {
        class_name_cap, // {}Error::Rclrs(error)
        class_name_cap, // let lib = {}::new();
        class_name_cap, // let lib = {}::with_name("test".to_string());
        class_name_cap  // let lib = {}::new();
    )
}
