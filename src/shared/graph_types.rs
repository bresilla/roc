use crate::shared::qos_profile::QosProfile;

/// Information about a topic endpoint (publisher or subscriber)
#[derive(Debug, Clone)]
pub struct TopicEndpointInfo {
    pub node_name: String,
    pub node_namespace: String,
    pub topic_type: String,
    pub topic_type_hash: String,
    pub endpoint_type: EndpointType,
    pub gid: Vec<u8>,
    pub qos_profile: QosProfile,
}

/// Endpoint type enum
#[derive(Debug, Clone)]
pub enum EndpointType {
    Publisher,
    Subscription,
    Invalid,
}

/// Information about a topic, including its name and types
#[derive(Debug, Clone)]
pub struct TopicInfo {
    pub name: String,
    pub types: Vec<String>,
}

impl EndpointType {
    #[allow(non_upper_case_globals)]
    pub fn from_rmw(endpoint_type: rclrs::rmw_endpoint_type_t) -> Self {
        use rclrs::*;
        match endpoint_type {
            rmw_endpoint_type_e_RMW_ENDPOINT_PUBLISHER => EndpointType::Publisher,
            rmw_endpoint_type_e_RMW_ENDPOINT_SUBSCRIPTION => EndpointType::Subscription,
            _ => EndpointType::Invalid,
        }
    }
}