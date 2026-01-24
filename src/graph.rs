//! ROS 2 Graph API
//!
//! This module provides a small, safe wrapper around `rclrs` graph APIs.
//! It replaces the previous raw `rcl`/`rmw` usage.

pub use crate::shared::action_operations;
pub use crate::shared::dynamic_messages::DynamicSubscriber;
pub use crate::shared::graph_context::RclGraphContext;
pub use crate::shared::graph_types::{TopicEndpointInfo, TopicInfo};

pub use crate::shared::node_operations;
pub use crate::shared::service_operations;
pub use crate::shared::topic_operations;

use anyhow::Result;

impl RclGraphContext {
    /// Get all topics in the ROS graph.
    pub fn get_topic_names(&self) -> Result<Vec<String>> {
        topic_operations::get_topic_names(self)
    }

    /// Get all topics with their type information.
    pub fn get_topics_with_types(&self) -> Result<Vec<TopicInfo>> {
        topic_operations::get_topics_with_types(self)
    }

    /// Get all topics and their types as tuples.
    pub fn get_topic_names_and_types(&self) -> Result<Vec<(String, String)>> {
        topic_operations::get_topic_names_and_types(self)
    }

    /// Count the number of publishers for a given topic.
    pub fn count_publishers(&self, topic_name: &str) -> Result<usize> {
        topic_operations::count_publishers(self, topic_name)
    }

    /// Count the number of subscriptions for a given topic.
    pub fn count_subscribers(&self, topic_name: &str) -> Result<usize> {
        topic_operations::count_subscribers(self, topic_name)
    }

    /// Get publisher endpoint information for a topic.
    pub fn get_publishers_info(&self, topic_name: &str) -> Result<Vec<TopicEndpointInfo>> {
        topic_operations::get_publishers_info(self, topic_name)
    }

    /// Get subscription endpoint information for a topic.
    pub fn get_subscribers_info(&self, topic_name: &str) -> Result<Vec<TopicEndpointInfo>> {
        topic_operations::get_subscribers_info(self, topic_name)
    }

    /// Get all nodes in the ROS graph.
    #[allow(dead_code)]
    pub fn get_node_names(&self) -> Result<Vec<String>> {
        node_operations::get_node_names(self)
    }

    /// Get all nodes with their namespaces.
    #[allow(dead_code)]
    pub fn get_node_names_with_namespaces(&self) -> Result<Vec<(String, String)>> {
        node_operations::get_node_names_with_namespaces(self)
    }

    /// Get all services in the ROS graph.
    #[allow(dead_code)]
    pub fn get_service_names(&self) -> Result<Vec<String>> {
        service_operations::get_service_names(self)
    }

    /// Get all services and their types as tuples.
    #[allow(dead_code)]
    pub fn get_service_names_and_types(&self) -> Result<Vec<(String, String)>> {
        service_operations::get_service_names_and_types(self)
    }

    /// Create a dynamic subscription for any message type.
    pub fn create_subscription(
        &self,
        topic_name: &str,
        message_type: &str,
    ) -> Result<DynamicSubscriber> {
        // For now this uses an internal node+executor dedicated to the subscription.
        // Later we can share the graph node and executor to reduce DDS entities.
        let _ = self; // keep API compatible
        DynamicSubscriber::new(topic_name, message_type)
    }
}
