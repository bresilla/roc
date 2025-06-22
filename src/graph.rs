//! ROS 2 Graph API
//! 
//! This module provides a high-level interface to the ROS 2 graph using the modular
//! shared components. It re-exports the main types and provides convenience methods.

// Re-export the main components from shared modules
pub use crate::shared::graph_context::RclGraphContext;
pub use crate::shared::graph_types::{TopicInfo, TopicEndpointInfo, EndpointType};
pub use crate::shared::dynamic_messages::{
    DynamicMessageType, DynamicMessageRegistry, DynamicMessageIntrospection, MessageMemberInfo,
    is_message_type_available, get_available_message_types
};
pub use crate::shared::dynamic_messages::yaml_parser::{YamlValue, parse_yaml_message, validate_message_structure};
pub use crate::shared::dynamic_messages::serialization::{SerializedMessage, serialize_message, deserialize_message};
pub use crate::shared::dynamic_messages::generic_serialization;

// Re-export operation modules for direct access if needed
pub use crate::shared::topic_operations;
pub use crate::shared::node_operations;
pub use crate::shared::service_operations;

use anyhow::Result;
use rclrs::rosidl_message_type_support_t;

impl RclGraphContext {
    // Convenience methods that delegate to the operation modules
    // This provides a clean API while keeping the implementation modular

    /// Get all topics in the ROS graph
    pub fn get_topic_names(&self) -> Result<Vec<String>> {
        topic_operations::get_topic_names(self)
    }

    /// Get all topics with their type information
    pub fn get_topics_with_types(&self) -> Result<Vec<TopicInfo>> {
        topic_operations::get_topics_with_types(self)
    }

    /// Get all topics and their types as tuples
    pub fn get_topic_names_and_types(&self) -> Result<Vec<(String, String)>> {
        topic_operations::get_topic_names_and_types(self)
    }

    /// Count the number of publishers for a given topic
    pub fn count_publishers(&self, topic_name: &str) -> Result<usize> {
        topic_operations::count_publishers(self, topic_name)
    }

    /// Count the number of subscribers for a given topic
    pub fn count_subscribers(&self, topic_name: &str) -> Result<usize> {
        topic_operations::count_subscribers(self, topic_name)
    }

    /// Get detailed information about all publishers to a topic
    pub fn get_publishers_info(&self, topic_name: &str) -> Result<Vec<TopicEndpointInfo>> {
        topic_operations::get_publishers_info(self, topic_name)
    }

    /// Get detailed information about all subscribers to a topic
    pub fn get_subscribers_info(&self, topic_name: &str) -> Result<Vec<TopicEndpointInfo>> {
        topic_operations::get_subscribers_info(self, topic_name)
    }

    /// Get all nodes in the ROS graph
    #[allow(dead_code)]
    pub fn get_node_names(&self) -> Result<Vec<String>> {
        node_operations::get_node_names(self)
    }

    /// Get all nodes with their namespaces
    #[allow(dead_code)]
    pub fn get_node_names_with_namespaces(&self) -> Result<Vec<(String, String)>> {
        node_operations::get_node_names_with_namespaces(self)
    }

    /// Get all services in the ROS graph
    #[allow(dead_code)]
    pub fn get_service_names(&self) -> Result<Vec<String>> {
        service_operations::get_service_names(self)
    }

    /// Get all services and their types as tuples
    #[allow(dead_code)]
    pub fn get_service_names_and_types(&self) -> Result<Vec<(String, String)>> {
        service_operations::get_service_names_and_types(self)
    }

    /// Create a new dynamic message registry
    pub fn create_message_registry() -> DynamicMessageRegistry {
        DynamicMessageRegistry::new()
    }

    /// Parse and validate a YAML message for a given message type
    pub fn parse_and_validate_message(message_type: &str, yaml_content: &str) -> Result<YamlValue> {
        // Parse the YAML content
        let yaml_value = parse_yaml_message(yaml_content)?;
        
        // Validate the structure for known message types
        validate_message_structure(message_type, &yaml_value)?;
        
        Ok(yaml_value)
    }

    /// Check if a message type is supported
    pub fn is_message_type_supported(message_type: &str) -> bool {
        is_message_type_available(message_type)
    }

    /// Get available message types for a package
    pub fn get_package_message_types(package_name: &str) -> Vec<String> {
        get_available_message_types(package_name)
    }

    /// Parse, validate, and serialize a YAML message for publishing
    pub fn prepare_message_for_publishing(
        message_type: &str, 
        yaml_content: &str
    ) -> Result<SerializedMessage> {
        // Parse and validate the YAML content
        let yaml_value = Self::parse_and_validate_message(message_type, yaml_content)?;
        
        // Serialize to binary format using our current approach
        // TODO: Use generic_serialization when type support is available
        serialize_message(message_type, &yaml_value)
    }

    /// Parse, validate, and serialize using introspection (when type support is available)
    pub fn prepare_message_for_publishing_generic(
        message_type: &str, 
        yaml_content: &str,
        type_support: *const rosidl_message_type_support_t,
    ) -> Result<SerializedMessage> {
        // Parse and validate the YAML content
        let yaml_value = Self::parse_and_validate_message(message_type, yaml_content)?;
        
        // Try generic serialization first, fall back to manual if needed
        match generic_serialization::serialize_message_generic(message_type, &yaml_value, type_support) {
            Ok(result) => {
                println!("Successfully used generic introspection-based serialization!");
                Ok(result)
            }
            Err(e) => {
                println!("Generic serialization failed ({}), falling back to manual", e);
                serialize_message(message_type, &yaml_value)
            }
        }
    }

    /// Deserialize binary message data for inspection
    pub fn inspect_serialized_message(
        message_type: &str,
        data: &[u8]
    ) -> Result<YamlValue> {
        deserialize_message(message_type, data)
    }
}