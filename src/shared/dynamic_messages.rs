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
        let mut msg_type = DynamicMessageType {
            type_name: type_name.to_string(),
            package_name: package_name.clone(),
            message_name: message_name.clone(),
            type_support: None,
            introspection: None,
        };

        // Try to load type support using available RCL functions
        if let Ok(type_support) = self.try_get_type_support(&package_name, &message_name) {
            msg_type.type_support = Some(type_support);
            
            // Try to get introspection data
            if let Ok(introspection) = self.try_get_message_introspection(type_support) {
                msg_type.introspection = Some(introspection);
            }
            
            println!("Successfully loaded type support for {}", type_name);
        } else {
            println!("Warning: Could not load dynamic type support for {} - using basic validation only", type_name);
        }
        
        // Cache the result
        let result = msg_type.clone();
        self.loaded_types.insert(type_name.to_string(), msg_type);
        
        Ok(result)
    }

    /// Test dynamic type support loading (for debugging)
    pub fn test_type_support_loading(&mut self) -> Result<()> {
        println!("Testing dynamic type support loading...");
        
        match self.load_message_type("geometry_msgs/msg/Twist") {
            Ok(msg_type) => {
                if msg_type.type_support.is_some() {
                    println!("✅ Successfully loaded type support for geometry_msgs/msg/Twist");
                } else {
                    println!("❌ Type support is None for geometry_msgs/msg/Twist");
                }
            }
            Err(e) => {
                println!("❌ Failed to load geometry_msgs/msg/Twist: {}", e);
                return Err(e);
            }
        }
        
        Ok(())
    }

    /// Try to get type support using available RCL mechanisms
    /// 
    /// This implements a step-by-step approach to find type support
    fn try_get_type_support(
        &self,
        package_name: &str,
        message_name: &str,
    ) -> Result<*const rosidl_message_type_support_t> {
        // For common message types, we can try to get their type support directly
        // This is a practical approach while we work on full dynamic loading
        match format!("{}/msg/{}", package_name, message_name).as_str() {
            "geometry_msgs/msg/Twist" => self.try_get_twist_type_support(),
            "std_msgs/msg/String" => self.try_get_string_type_support(),
            "std_msgs/msg/Int32" => self.try_get_int32_type_support(),
            "std_msgs/msg/Float64" => self.try_get_float64_type_support(),
            _ => {
                // For unknown types, return an error for now
                // This can be expanded with actual dynamic loading later
                Err(anyhow!("Dynamic type support not yet implemented for {}/{}", package_name, message_name))
            }
        }
    }

    /// Try to get type support for geometry_msgs/msg/Twist
    fn try_get_twist_type_support(&self) -> Result<*const rosidl_message_type_support_t> {
        // Load the geometry_msgs typesupport library dynamically
        let library_path = "/opt/ros/jazzy/lib/libgeometry_msgs__rosidl_typesupport_c.so";
        let symbol_name = "rosidl_typesupport_c__get_message_type_support_handle__geometry_msgs__msg__Twist";
        
        self.load_type_support_from_library(library_path, symbol_name)
    }

    /// Try to get type support for std_msgs/msg/String
    fn try_get_string_type_support(&self) -> Result<*const rosidl_message_type_support_t> {
        // Placeholder implementation
        Err(anyhow!("String type support loading not yet implemented"))
    }

    /// Try to get type support for std_msgs/msg/Int32
    fn try_get_int32_type_support(&self) -> Result<*const rosidl_message_type_support_t> {
        // Placeholder implementation
        Err(anyhow!("Int32 type support loading not yet implemented"))
    }

    /// Try to get type support for std_msgs/msg/Float64
    fn try_get_float64_type_support(&self) -> Result<*const rosidl_message_type_support_t> {
        // Placeholder implementation
        Err(anyhow!("Float64 type support loading not yet implemented"))
    }

    /// Load type support from a shared library
    fn load_type_support_from_library(
        &self,
        library_name: &str,
        symbol_name: &str,
    ) -> Result<*const rosidl_message_type_support_t> {
        use std::ffi::CString;
        
        unsafe {
            // Initialize a shared library handle
            let mut shared_lib = rcutils_get_zero_initialized_shared_library();
            
            // Convert library name to C string
            let lib_name_c = CString::new(library_name)
                .map_err(|e| anyhow!("Invalid library name '{}': {}", library_name, e))?;
            
            // Load the shared library
            let allocator = rcutils_get_default_allocator();
            let ret = rcutils_load_shared_library(
                &mut shared_lib,
                lib_name_c.as_ptr(),
                allocator,
            );
            
            if ret != 0 { // RCUTILS_RET_OK is 0
                return Err(anyhow!("Failed to load library '{}': return code {}", library_name, ret));
            }
            
            // Convert symbol name to C string
            let symbol_name_c = CString::new(symbol_name)
                .map_err(|e| anyhow!("Invalid symbol name '{}': {}", symbol_name, e))?;
            
            // Get the symbol from the library
            let symbol_ptr = rcutils_get_symbol(&shared_lib, symbol_name_c.as_ptr());
            
            if symbol_ptr.is_null() {
                rcutils_unload_shared_library(&mut shared_lib);
                return Err(anyhow!("Symbol '{}' not found in library '{}'", symbol_name, library_name));
            }
            
            // Cast the symbol to a function pointer and call it
            type TypeSupportGetterFn = unsafe extern "C" fn() -> *const rosidl_message_type_support_t;
            let type_support_fn: TypeSupportGetterFn = std::mem::transmute(symbol_ptr);
            let type_support = type_support_fn();
            
            // Note: We intentionally don't unload the library here, as the type support
            // pointer would become invalid. In a production system, we'd need to track
            // loaded libraries and unload them during cleanup.
            
            if type_support.is_null() {
                return Err(anyhow!("Type support function returned null pointer"));
            }
            
            println!("Successfully loaded type support for symbol: {}", symbol_name);
            Ok(type_support)
        }
    }

    /// Try to get message introspection data from type support
    fn try_get_message_introspection(
        &self,
        _type_support: *const rosidl_message_type_support_t,
    ) -> Result<DynamicMessageIntrospection> {
        // This would use the introspection functions once they're properly bound
        // For now, return a basic introspection structure
        Err(anyhow!("Message introspection not yet implemented"))
    }

    /// Get basic information about a message type
    pub fn get_message_info(&self, type_name: &str) -> Result<String> {
        if let Some(msg_type) = self.loaded_types.get(type_name) {
            let type_support_status = if msg_type.type_support.is_some() {
                "loaded"
            } else {
                "not loaded"
            };
            
            Ok(format!(
                "Message type: {} (Package: {}, Type support: {})", 
                msg_type.message_name, 
                msg_type.package_name,
                type_support_status
            ))
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

/// Message serialization support for converting between YAML and binary formats
pub mod serialization {
    use super::*;
    use super::yaml_parser::YamlValue;
    
    /// Serialized message data
    #[derive(Debug, Clone)]
    pub struct SerializedMessage {
        pub data: Vec<u8>,
        pub message_type: String,
    }
    
    /// Serialize a YAML message to binary format for ROS2 publishing
    /// 
    /// This converts the parsed YAML structure to the binary format expected by ROS2
    pub fn serialize_message(
        message_type: &str,
        yaml_value: &YamlValue,
    ) -> Result<SerializedMessage> {
        match message_type {
            "geometry_msgs/msg/Twist" => serialize_twist_message(yaml_value),
            "std_msgs/msg/String" => serialize_string_message(yaml_value),
            "std_msgs/msg/Int32" => serialize_int32_message(yaml_value),
            "std_msgs/msg/Float64" => serialize_float64_message(yaml_value),
            _ => Err(anyhow!("Serialization not implemented for message type: {}", message_type)),
        }
    }
    
    /// Serialize geometry_msgs/msg/Twist message
    fn serialize_twist_message(yaml_value: &YamlValue) -> Result<SerializedMessage> {
        if let YamlValue::Object(obj) = yaml_value {
            // Extract linear and angular components
            let linear = extract_vector3(obj.get("linear"), "linear")?;
            let angular = extract_vector3(obj.get("angular"), "angular")?;
            
            // Create binary representation
            // This is a simplified binary format - real ROS2 uses CDR serialization
            let mut data = Vec::new();
            
            // Linear vector (24 bytes: 3 x f64)
            data.extend_from_slice(&linear.0.to_le_bytes());
            data.extend_from_slice(&linear.1.to_le_bytes());
            data.extend_from_slice(&linear.2.to_le_bytes());
            
            // Angular vector (24 bytes: 3 x f64)
            data.extend_from_slice(&angular.0.to_le_bytes());
            data.extend_from_slice(&angular.1.to_le_bytes());
            data.extend_from_slice(&angular.2.to_le_bytes());
            
            Ok(SerializedMessage {
                data,
                message_type: "geometry_msgs/msg/Twist".to_string(),
            })
        } else {
            Err(anyhow!("Twist message must be an object"))
        }
    }
    
    /// Extract Vector3 (x, y, z) from YAML object
    fn extract_vector3(yaml_value: Option<&YamlValue>, field_name: &str) -> Result<(f64, f64, f64)> {
        if let Some(YamlValue::Object(obj)) = yaml_value {
            let x = extract_number(obj.get("x"), &format!("{}.x", field_name))?;
            let y = extract_number(obj.get("y"), &format!("{}.y", field_name))?;
            let z = extract_number(obj.get("z"), &format!("{}.z", field_name))?;
            Ok((x, y, z))
        } else {
            // Default to zero if not specified
            Ok((0.0, 0.0, 0.0))
        }
    }
    
    /// Extract a number from YAML value
    fn extract_number(yaml_value: Option<&YamlValue>, field_name: &str) -> Result<f64> {
        match yaml_value {
            Some(YamlValue::Float(f)) => Ok(*f),
            Some(YamlValue::Int(i)) => Ok(*i as f64),
            Some(_) => Err(anyhow!("{} must be a number", field_name)),
            None => Ok(0.0), // Default to 0 if not specified
        }
    }
    
    /// Serialize std_msgs/msg/String message
    fn serialize_string_message(yaml_value: &YamlValue) -> Result<SerializedMessage> {
        if let YamlValue::Object(obj) = yaml_value {
            if let Some(YamlValue::String(data)) = obj.get("data") {
                // String serialization: length (4 bytes) + string data
                let string_bytes = data.as_bytes();
                let mut serialized_data = Vec::new();
                
                // Add string length as 4-byte little-endian
                serialized_data.extend_from_slice(&(string_bytes.len() as u32).to_le_bytes());
                // Add string data
                serialized_data.extend_from_slice(string_bytes);
                
                Ok(SerializedMessage {
                    data: serialized_data,
                    message_type: "std_msgs/msg/String".to_string(),
                })
            } else {
                Err(anyhow!("String message must have a 'data' field with string value"))
            }
        } else {
            Err(anyhow!("String message must be an object"))
        }
    }
    
    /// Serialize std_msgs/msg/Int32 message
    fn serialize_int32_message(yaml_value: &YamlValue) -> Result<SerializedMessage> {
        if let YamlValue::Object(obj) = yaml_value {
            if let Some(data_value) = obj.get("data") {
                let data = match data_value {
                    YamlValue::Int(i) => *i as i32,
                    YamlValue::Float(f) => *f as i32,
                    _ => return Err(anyhow!("Int32 message data must be a number")),
                };
                
                Ok(SerializedMessage {
                    data: data.to_le_bytes().to_vec(),
                    message_type: "std_msgs/msg/Int32".to_string(),
                })
            } else {
                Err(anyhow!("Int32 message must have a 'data' field"))
            }
        } else {
            Err(anyhow!("Int32 message must be an object"))
        }
    }
    
    /// Serialize std_msgs/msg/Float64 message
    fn serialize_float64_message(yaml_value: &YamlValue) -> Result<SerializedMessage> {
        if let YamlValue::Object(obj) = yaml_value {
            if let Some(data_value) = obj.get("data") {
                let data = match data_value {
                    YamlValue::Float(f) => *f,
                    YamlValue::Int(i) => *i as f64,
                    _ => return Err(anyhow!("Float64 message data must be a number")),
                };
                
                Ok(SerializedMessage {
                    data: data.to_le_bytes().to_vec(),
                    message_type: "std_msgs/msg/Float64".to_string(),
                })
            } else {
                Err(anyhow!("Float64 message must have a 'data' field"))
            }
        } else {
            Err(anyhow!("Float64 message must be an object"))
        }
    }
    
    /// Deserialize binary message data back to YAML format
    /// 
    /// This is useful for debugging and message inspection
    pub fn deserialize_message(
        message_type: &str,
        data: &[u8],
    ) -> Result<YamlValue> {
        match message_type {
            "geometry_msgs/msg/Twist" => deserialize_twist_message(data),
            "std_msgs/msg/String" => deserialize_string_message(data),
            "std_msgs/msg/Int32" => deserialize_int32_message(data),
            "std_msgs/msg/Float64" => deserialize_float64_message(data),
            _ => Err(anyhow!("Deserialization not implemented for message type: {}", message_type)),
        }
    }
    
    /// Deserialize geometry_msgs/msg/Twist message
    fn deserialize_twist_message(data: &[u8]) -> Result<YamlValue> {
        if data.len() != 48 { // 6 x f64 = 48 bytes
            return Err(anyhow!("Invalid Twist message size: expected 48 bytes, got {}", data.len()));
        }
        
        // Extract the 6 f64 values
        let mut values = Vec::new();
        for i in 0..6 {
            let start = i * 8;
            let bytes: [u8; 8] = data[start..start + 8].try_into()
                .map_err(|_| anyhow!("Failed to extract f64 at position {}", i))?;
            values.push(f64::from_le_bytes(bytes));
        }
        
        let mut twist_obj = HashMap::new();
        
        // Linear component
        let mut linear_obj = HashMap::new();
        linear_obj.insert("x".to_string(), YamlValue::Float(values[0]));
        linear_obj.insert("y".to_string(), YamlValue::Float(values[1]));
        linear_obj.insert("z".to_string(), YamlValue::Float(values[2]));
        twist_obj.insert("linear".to_string(), YamlValue::Object(linear_obj));
        
        // Angular component
        let mut angular_obj = HashMap::new();
        angular_obj.insert("x".to_string(), YamlValue::Float(values[3]));
        angular_obj.insert("y".to_string(), YamlValue::Float(values[4]));
        angular_obj.insert("z".to_string(), YamlValue::Float(values[5]));
        twist_obj.insert("angular".to_string(), YamlValue::Object(angular_obj));
        
        Ok(YamlValue::Object(twist_obj))
    }
    
    /// Deserialize std_msgs/msg/String message
    fn deserialize_string_message(data: &[u8]) -> Result<YamlValue> {
        if data.len() < 4 {
            return Err(anyhow!("Invalid String message: too short"));
        }
        
        // Extract string length
        let length_bytes: [u8; 4] = data[0..4].try_into()
            .map_err(|_| anyhow!("Failed to extract string length"))?;
        let length = u32::from_le_bytes(length_bytes) as usize;
        
        if data.len() != 4 + length {
            return Err(anyhow!("Invalid String message size: expected {} bytes, got {}", 4 + length, data.len()));
        }
        
        // Extract string data
        let string_data = String::from_utf8(data[4..].to_vec())
            .map_err(|e| anyhow!("Invalid UTF-8 in string message: {}", e))?;
        
        let mut obj = HashMap::new();
        obj.insert("data".to_string(), YamlValue::String(string_data));
        
        Ok(YamlValue::Object(obj))
    }
    
    /// Deserialize std_msgs/msg/Int32 message
    fn deserialize_int32_message(data: &[u8]) -> Result<YamlValue> {
        if data.len() != 4 {
            return Err(anyhow!("Invalid Int32 message size: expected 4 bytes, got {}", data.len()));
        }
        
        let bytes: [u8; 4] = data.try_into()
            .map_err(|_| anyhow!("Failed to extract Int32 data"))?;
        let value = i32::from_le_bytes(bytes);
        
        let mut obj = HashMap::new();
        obj.insert("data".to_string(), YamlValue::Int(value as i64));
        
        Ok(YamlValue::Object(obj))
    }
    
    /// Deserialize std_msgs/msg/Float64 message
    fn deserialize_float64_message(data: &[u8]) -> Result<YamlValue> {
        if data.len() != 8 {
            return Err(anyhow!("Invalid Float64 message size: expected 8 bytes, got {}", data.len()));
        }
        
        let bytes: [u8; 8] = data.try_into()
            .map_err(|_| anyhow!("Failed to extract Float64 data"))?;
        let value = f64::from_le_bytes(bytes);
        
        let mut obj = HashMap::new();
        obj.insert("data".to_string(), YamlValue::Float(value));
        
        Ok(YamlValue::Object(obj))
    }
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