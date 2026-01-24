/// Information about a topic endpoint (publisher or subscription).
///
/// Note: `rclrs` (0.7.x) only exposes a limited endpoint info set via its safe
/// graph API: node name, node namespace, and topic type.
///
/// Fields like QoS profile, endpoint GID, type hash, and endpoint kind are not
/// available without dropping down into the raw `rcl`/`rmw` APIs.
#[derive(Debug, Clone)]
pub struct TopicEndpointInfo {
    pub node_name: String,
    pub node_namespace: String,
    pub topic_type: String,
}

/// Information about a topic, including its name and types
#[derive(Debug, Clone)]
pub struct TopicInfo {
    pub name: String,
    pub types: Vec<String>,
}
