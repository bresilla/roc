//! ROS 2 Graph API
//! 
//! This module provides a high-level interface to the ROS 2 graph using the modular
//! shared components. It re-exports the main types and provides convenience methods.

// Re-export the main components from shared modules
pub use crate::shared::graph_context::RclGraphContext;
pub use crate::shared::graph_types::{TopicInfo, TopicEndpointInfo, EndpointType};

// Re-export operation modules for direct access if needed
pub use crate::shared::topic_operations;
pub use crate::shared::node_operations;
pub use crate::shared::service_operations;

use anyhow::Result;

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
}