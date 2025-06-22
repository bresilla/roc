use rclrs::*;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_yaml;

/// Dynamic message type support infrastructure for ROS2
/// 
/// This module provides the foundation for loading and working with ROS2 message types
/// dynamically at runtime without compile-time knowledge of the message structure.

#[derive(Debug, Clone)]
pub struct DynamicMessageType {
    /// The full message type name (e.g., "geometry_msgs/msg/Twist")
    pub type_name: String,
    /// Package name extracted from type_name
    pub package_name: String,
    /// Message name extracted from type_name  
    pub message_name: String,
    /// Pointer to the type support structure (if loaded)
    pub type_support: Option<*const rosidl_message_type_support_t>,
    /// Message introspection data (if available)
    pub introspection: Option<DynamicMessageIntrospection>,
}

#[derive(Debug, Clone)]
pub struct DynamicMessageIntrospection {
    /// Message size in bytes
    pub size_of: usize,
    /// Alignment requirements
    pub alignment_of: usize,
    /// Number of fields in the message
    pub member_count: u32,
    /// Field information
    pub members: Vec<MessageMemberInfo>,
}

#[derive(Debug, Clone)]
pub struct MessageMemberInfo {
    /// Field name
    pub name: String,
    /// Field type ID 
    pub type_id: u8,
    /// Whether this field is an array
    pub is_array: bool,
    /// Array size (0 for dynamic arrays)
    pub array_size: usize,
    /// Offset within the message structure
    pub offset: usize,
    /// Whether this field is optional
    pub is_optional: bool,
}

/// Dynamic message type registry
pub struct DynamicMessageRegistry {
    /// Cache of loaded message types
    loaded_types: HashMap<String, DynamicMessageType>,
}

impl DynamicMessageRegistry {
    /// Create a new message type registry
    pub fn new() -> Self {
        Self {
            loaded_types: HashMap::new(),
        }
    }

    /// Parse a message type string into components
    /// 
    /// Example: "geometry_msgs/msg/Twist" -> ("geometry_msgs", "Twist")
    pub fn parse_message_type(type_name: &str) -> Result<(String, String)> {
        let parts: Vec<&str> = type_name.split('/').collect();
        if parts.len() != 3 || parts[1] != "msg" {
            return Err(anyhow!(
                "Invalid message type format. Expected 'package/msg/MessageName', got '{}'", 
                type_name
            ));
        }
        
        Ok((parts[0].to_string(), parts[2].to_string()))
    }

    /// Attempt to load dynamic type support for a message type
    /// 
    /// This tries multiple strategies to load the type support:
    /// 1. Check if already loaded in cache
    /// 2. Try to load from shared library
    /// 3. Try to get introspection type support
    pub fn load_message_type(&mut self, type_name: &str) -> Result<DynamicMessageType> {
        // Check cache first
        if let Some(msg_type) = self.loaded_types.get(type_name) {
            return Ok(msg_type.clone());
        }

        let (package_name, message_name) = Self::parse_message_type(type_name)?;
        
        println!("Loading dynamic type support for {}/{}", package_name, message_name);
        
        // Create the message type structure
        let msg_type = DynamicMessageType {
            type_name: type_name.to_string(),
            package_name: package_name.clone(),
            message_name: message_name.clone(),
            type_support: None,
            introspection: None,
        };

        // For now, create a placeholder implementation
        // TODO: Implement actual type support loading once all RCL functions are available
        println!("Dynamic type support loading not yet fully implemented for {}", type_name);
        
        // Cache the result (without type support for now)
        let result = msg_type.clone();
        self.loaded_types.insert(type_name.to_string(), msg_type);
        
        Ok(result)
    }

    /// Get basic information about a message type (placeholder implementation)
    /// 
    /// This will be expanded once all RCL functions are available
    pub fn get_message_info(&self, type_name: &str) -> Result<String> {
        if let Some(msg_type) = self.loaded_types.get(type_name) {
            Ok(format!("Message type: {} (Package: {})", msg_type.message_name, msg_type.package_name))
        } else {
            Err(anyhow!("Message type '{}' not loaded", type_name))
        }
    }
}

/// Check if a message type is available in the system
/// 
/// This is a basic validation check for message type format
pub fn is_message_type_available(type_name: &str) -> bool {
    DynamicMessageRegistry::parse_message_type(type_name).is_ok()
}

/// Get a list of available message types in a package
/// 
/// This scans the file system for available type support libraries
pub fn get_available_message_types(package_name: &str) -> Vec<String> {
    // This is a placeholder implementation
    // In a full implementation, this would scan the ROS2 installation
    // for available message type libraries and extract the message names
    let mut types = Vec::new();
    
    // For now, return some common types as examples
    match package_name {
        "geometry_msgs" => {
            types.extend_from_slice(&[
                "geometry_msgs/msg/Twist".to_string(),
                "geometry_msgs/msg/Pose".to_string(),
                "geometry_msgs/msg/Point".to_string(),
                "geometry_msgs/msg/Vector3".to_string(),
            ]);
        },
        "std_msgs" => {
            types.extend_from_slice(&[
                "std_msgs/msg/String".to_string(),
                "std_msgs/msg/Int32".to_string(),
                "std_msgs/msg/Float64".to_string(),
                "std_msgs/msg/Bool".to_string(),
            ]);
        },
        _ => {
            // Could implement directory scanning here
        }
    }
    
    types
}

/// YAML message parsing and validation
pub mod yaml_parser {
    use super::*;
    
    /// Generic YAML value that can represent any ROS message field
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum YamlValue {
        Bool(bool),
        Int(i64),
        Float(f64),
        String(String),
        Array(Vec<YamlValue>),
        Object(HashMap<String, YamlValue>),
    }
    
    /// Parse a YAML string into a structured message representation
    /// 
    /// This handles both simple values and complex nested structures
    /// Example: "{linear: {x: 0.5, y: 0.0, z: 0.0}, angular: {x: 0.0, y: 0.0, z: 0.0}}"
    pub fn parse_yaml_message(yaml_content: &str) -> Result<YamlValue> {
        // Handle both YAML and simplified formats
        let processed_content = if yaml_content.trim().starts_with('{') && yaml_content.trim().ends_with('}') {
            // Convert simplified format to proper YAML
            convert_simplified_to_yaml(yaml_content)?
        } else {
            yaml_content.to_string()
        };
        
        let value: serde_yaml::Value = serde_yaml::from_str(&processed_content)
            .map_err(|e| anyhow!("Failed to parse YAML: {}", e))?;
        
        convert_serde_yaml_to_yaml_value(value)
    }
    
    /// Convert simplified ROS message format to proper YAML
    /// 
    /// Converts: {linear: {x: 0.5}} -> "linear:\n  x: 0.5"
    fn convert_simplified_to_yaml(content: &str) -> Result<String> {
        // Remove outer braces
        let inner = content.trim().strip_prefix('{').and_then(|s| s.strip_suffix('}')).unwrap_or(content);
        
        // For now, try to parse as YAML directly
        // This is a simplified implementation - a full implementation would handle
        // more complex parsing of the ROS message format
        
        if inner.trim().is_empty() {
            return Ok("{}".to_string());
        }
        
        // Try to parse as YAML first
        if let Ok(_) = serde_yaml::from_str::<serde_yaml::Value>(&format!("{{{}}}", inner)) {
            return Ok(format!("{{{}}}", inner));
        }
        
        // Basic conversion for simple cases
        let yaml_format = inner
            .replace(": {", ":\n  ")
            .replace(", ", "\n")
            .replace("}", "");
            
        Ok(yaml_format)
    }
    
    /// Convert serde_yaml::Value to our YamlValue
    fn convert_serde_yaml_to_yaml_value(value: serde_yaml::Value) -> Result<YamlValue> {
        match value {
            serde_yaml::Value::Bool(b) => Ok(YamlValue::Bool(b)),
            serde_yaml::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(YamlValue::Int(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(YamlValue::Float(f))
                } else {
                    Err(anyhow!("Invalid number format"))
                }
            },
            serde_yaml::Value::String(s) => Ok(YamlValue::String(s)),
            serde_yaml::Value::Sequence(seq) => {
                let mut array = Vec::new();
                for item in seq {
                    array.push(convert_serde_yaml_to_yaml_value(item)?);
                }
                Ok(YamlValue::Array(array))
            },
            serde_yaml::Value::Mapping(map) => {
                let mut object = HashMap::new();
                for (k, v) in map {
                    if let serde_yaml::Value::String(key) = k {
                        object.insert(key, convert_serde_yaml_to_yaml_value(v)?);
                    } else {
                        return Err(anyhow!("Non-string keys not supported"));
                    }
                }
                Ok(YamlValue::Object(object))
            },
            serde_yaml::Value::Null => Ok(YamlValue::Object(HashMap::new())),
            _ => Err(anyhow!("Unsupported YAML value type")),
        }
    }
    
    /// Validate message structure against expected fields
    /// 
    /// This is a basic validation - will be expanded with actual message introspection
    pub fn validate_message_structure(
        message_type: &str,
        yaml_value: &YamlValue,
    ) -> Result<()> {
        match message_type {
            "geometry_msgs/msg/Twist" => validate_twist_message(yaml_value),
            "std_msgs/msg/String" => validate_string_message(yaml_value),
            "std_msgs/msg/Int32" => validate_int32_message(yaml_value),
            "std_msgs/msg/Float64" => validate_float64_message(yaml_value),
            _ => {
                println!("Warning: Unknown message type '{}', skipping validation", message_type);
                Ok(())
            }
        }
    }
    
    /// Validate geometry_msgs/msg/Twist structure
    fn validate_twist_message(yaml_value: &YamlValue) -> Result<()> {
        if let YamlValue::Object(obj) = yaml_value {
            // Check for required fields
            if let Some(linear) = obj.get("linear") {
                validate_vector3(linear, "linear")?;
            }
            if let Some(angular) = obj.get("angular") {
                validate_vector3(angular, "angular")?;
            }
            Ok(())
        } else {
            Err(anyhow!("Twist message must be an object"))
        }
    }
    
    /// Validate Vector3 structure (x, y, z fields)
    fn validate_vector3(yaml_value: &YamlValue, field_name: &str) -> Result<()> {
        if let YamlValue::Object(obj) = yaml_value {
            for axis in &["x", "y", "z"] {
                if let Some(value) = obj.get(*axis) {
                    match value {
                        YamlValue::Float(_) | YamlValue::Int(_) => {},
                        _ => return Err(anyhow!("{}.{} must be a number", field_name, axis)),
                    }
                }
            }
            Ok(())
        } else {
            Err(anyhow!("{} must be an object with x, y, z fields", field_name))
        }
    }
    
    /// Validate std_msgs/msg/String structure
    fn validate_string_message(yaml_value: &YamlValue) -> Result<()> {
        if let YamlValue::Object(obj) = yaml_value {
            if let Some(data) = obj.get("data") {
                match data {
                    YamlValue::String(_) => Ok(()),
                    _ => Err(anyhow!("String message data field must be a string")),
                }
            } else {
                Err(anyhow!("String message must have a 'data' field"))
            }
        } else {
            Err(anyhow!("String message must be an object"))
        }
    }
    
    /// Validate std_msgs/msg/Int32 structure
    fn validate_int32_message(yaml_value: &YamlValue) -> Result<()> {
        if let YamlValue::Object(obj) = yaml_value {
            if let Some(data) = obj.get("data") {
                match data {
                    YamlValue::Int(_) => Ok(()),
                    _ => Err(anyhow!("Int32 message data field must be an integer")),
                }
            } else {
                Err(anyhow!("Int32 message must have a 'data' field"))
            }
        } else {
            Err(anyhow!("Int32 message must be an object"))
        }
    }
    
    /// Validate std_msgs/msg/Float64 structure
    fn validate_float64_message(yaml_value: &YamlValue) -> Result<()> {
        if let YamlValue::Object(obj) = yaml_value {
            if let Some(data) = obj.get("data") {
                match data {
                    YamlValue::Float(_) | YamlValue::Int(_) => Ok(()),
                    _ => Err(anyhow!("Float64 message data field must be a number")),
                }
            } else {
                Err(anyhow!("Float64 message must have a 'data' field"))
            }
        } else {
            Err(anyhow!("Float64 message must be an object"))
        }
    }
    
    /// Convert YamlValue back to YAML string for debugging
    pub fn yaml_value_to_string(value: &YamlValue) -> Result<String> {
        let serde_value = convert_yaml_value_to_serde_yaml(value)?;
        serde_yaml::to_string(&serde_value)
            .map_err(|e| anyhow!("Failed to serialize to YAML: {}", e))
    }
    
    /// Convert our YamlValue back to serde_yaml::Value
    fn convert_yaml_value_to_serde_yaml(value: &YamlValue) -> Result<serde_yaml::Value> {
        match value {
            YamlValue::Bool(b) => Ok(serde_yaml::Value::Bool(*b)),
            YamlValue::Int(i) => Ok(serde_yaml::Value::Number(serde_yaml::Number::from(*i))),
            YamlValue::Float(f) => Ok(serde_yaml::Value::Number(serde_yaml::Number::from(*f))),
            YamlValue::String(s) => Ok(serde_yaml::Value::String(s.clone())),
            YamlValue::Array(arr) => {
                let mut seq = Vec::new();
                for item in arr {
                    seq.push(convert_yaml_value_to_serde_yaml(item)?);
                }
                Ok(serde_yaml::Value::Sequence(seq))
            },
            YamlValue::Object(obj) => {
                let mut map = serde_yaml::Mapping::new();
                for (k, v) in obj {
                    map.insert(
                        serde_yaml::Value::String(k.clone()),
                        convert_yaml_value_to_serde_yaml(v)?
                    );
                }
                Ok(serde_yaml::Value::Mapping(map))
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::yaml_parser::*;

    #[test]
    fn test_parse_message_type() {
        let result = DynamicMessageRegistry::parse_message_type("geometry_msgs/msg/Twist");
        assert!(result.is_ok());
        let (package, message) = result.unwrap();
        assert_eq!(package, "geometry_msgs");
        assert_eq!(message, "Twist");
    }

    #[test]
    fn test_invalid_message_type() {
        let result = DynamicMessageRegistry::parse_message_type("invalid_format");
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_creation() {
        let registry = DynamicMessageRegistry::new();
        assert_eq!(registry.loaded_types.len(), 0);
    }

    #[test]
    fn test_yaml_parsing_simple() {
        let yaml_content = "{linear: {x: 0.5, y: 0.0, z: 0.0}}";
        let result = parse_yaml_message(yaml_content);
        assert!(result.is_ok());
        
        if let Ok(YamlValue::Object(obj)) = result {
            assert!(obj.contains_key("linear"));
        } else {
            panic!("Expected object result");
        }
    }

    #[test]
    fn test_twist_message_validation() {
        let yaml_content = "{linear: {x: 0.5, y: 0.0, z: 0.0}, angular: {x: 0.0, y: 0.0, z: 0.5}}";
        let yaml_value = parse_yaml_message(yaml_content).unwrap();
        let result = validate_message_structure("geometry_msgs/msg/Twist", &yaml_value);
        assert!(result.is_ok());
    }

    #[test]
    fn test_string_message_validation() {
        let yaml_content = "{data: 'hello world'}";
        let yaml_value = parse_yaml_message(yaml_content).unwrap();
        let result = validate_message_structure("std_msgs/msg/String", &yaml_value);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_twist_validation() {
        let yaml_content = "{linear: 'invalid'}";
        let yaml_value = parse_yaml_message(yaml_content).unwrap();
        let result = validate_message_structure("geometry_msgs/msg/Twist", &yaml_value);
        assert!(result.is_err());
    }
}