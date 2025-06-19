#![allow(non_upper_case_globals)]

use rclrs::*;
use std::ptr;
use std::ffi::CString;
use std::env;
use std::process::Command;
use std::net::TcpStream;
use anyhow::{Result, anyhow};

/// A simple RCL context manager for graph operations  
/// Includes both context and a minimal node for graph queries
pub struct RclGraphContext {
    context: rcl_context_t,
    node: rcl_node_t,
    is_initialized: bool,
}

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

/// QoS Profile information
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct QosProfile {
    pub history: QosHistoryPolicy,
    pub depth: usize,
    pub reliability: QosReliabilityPolicy,
    pub durability: QosDurabilityPolicy,
    pub deadline_sec: u64,
    pub deadline_nsec: u64,
    pub lifespan_sec: u64,
    pub lifespan_nsec: u64,
    pub liveliness: QosLivelinessPolicy,
    pub liveliness_lease_duration_sec: u64,
    pub liveliness_lease_duration_nsec: u64,
    pub avoid_ros_namespace_conventions: bool,
}

/// QoS History Policy
#[derive(Debug, Clone)]
pub enum QosHistoryPolicy {
    SystemDefault,
    KeepLast,
    KeepAll,
    Unknown,
}

/// QoS Reliability Policy
#[derive(Debug, Clone)]
pub enum QosReliabilityPolicy {
    SystemDefault,
    Reliable,
    BestEffort,
    Unknown,
    BestAvailable,
}

/// QoS Durability Policy
#[derive(Debug, Clone)]
pub enum QosDurabilityPolicy {
    SystemDefault,
    TransientLocal,
    Volatile,
    Unknown,
    BestAvailable,
}

/// QoS Liveliness Policy
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum QosLivelinessPolicy {
    SystemDefault,
    Automatic,
    ManualByNode,
    ManualByTopic,
    Unknown,
    BestAvailable,
}

impl RclGraphContext {
    /// Create a new RCL context for graph operations
    /// Note: This implementation always performs direct DDS discovery (equivalent to --no-daemon)
    pub fn new() -> Result<Self> {
        Self::new_with_discovery(std::time::Duration::from_millis(150))
    }

    /// Create a new RCL context for graph operations  
    /// Note: Our implementation is daemon-free by design, so this is identical to new()
    #[allow(dead_code)]
    pub fn new_no_daemon() -> Result<Self> {
        // Our implementation always does direct discovery, so this is the same as new()
        Self::new()
    }

    /// Create a new RCL context for graph operations with custom discovery time
    pub fn new_with_discovery(discovery_time: std::time::Duration) -> Result<Self> {
        unsafe {
            // Read ROS_DOMAIN_ID from environment (default to 0 if not set)
            let domain_id = env::var("ROS_DOMAIN_ID")
                .ok()
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);

            // Initialize RCL init options
            let mut init_options = rcl_get_zero_initialized_init_options();
            let allocator = rcutils_get_default_allocator();
            
            let ret = rcl_init_options_init(&mut init_options, allocator);
            if ret != 0 {
                return Err(anyhow!("Failed to initialize RCL init options: {}", ret));
            }

            // Get the RMW init options from RCL and set the domain ID
            let rmw_init_options = rcl_init_options_get_rmw_init_options(&mut init_options);
            if rmw_init_options.is_null() {
                return Err(anyhow!("Failed to get RMW init options"));
            }
            
            // Set the domain ID
            (*rmw_init_options).domain_id = domain_id;

            // Initialize RCL context with the configured options
            let mut context = rcl_get_zero_initialized_context();
            let ret = rcl_init(0, ptr::null_mut(), &init_options, &mut context);
            if ret != 0 {
                return Err(anyhow!("Failed to initialize RCL: {}", ret));
            }

            // Create a minimal node for graph queries
            let mut node = rcl_get_zero_initialized_node();
            let node_name = CString::new("roc_graph_node").map_err(|e| anyhow!("Failed to create node name: {}", e))?;
            let namespace = CString::new("/").map_err(|e| anyhow!("Failed to create namespace: {}", e))?;
            let node_options = rcl_node_get_default_options();
            
            let ret = rcl_node_init(
                &mut node,
                node_name.as_ptr(),
                namespace.as_ptr(),
                &mut context,
                &node_options,
            );
            if ret != 0 {
                rcl_shutdown(&mut context);
                return Err(anyhow!("Failed to initialize node: {}", ret));
            }

            let graph_context = RclGraphContext {
                context,
                node,
                is_initialized: true,
            };

            // Allow time for graph discovery
            graph_context.wait_for_graph_discovery(discovery_time)?;

            Ok(graph_context)
        }
    }

    /// Check if the context is valid
    pub fn is_valid(&self) -> bool {
        if !self.is_initialized {
            return false;
        }
        unsafe {
            rcl_context_is_valid(&self.context) && rcl_node_is_valid(&self.node)
        }
    }

    /// Wait for graph discovery with a reasonable timeout
    /// Since we always do direct DDS discovery, we need to allow time for DDS to discover peers
    fn wait_for_graph_discovery(&self, discovery_time: std::time::Duration) -> Result<()> {
        if !self.is_valid() {
            return Err(anyhow!("RCL context is not valid"));
        }

        // For direct DDS discovery, we need to wait for the network discovery protocol
        // This is necessary because DDS discovery is asynchronous
        std::thread::sleep(discovery_time);
        
        Ok(())
    }

    /// Get all topics in the ROS graph using direct RCL API calls
    pub fn get_topic_names(&self) -> Result<Vec<String>> {
        if !self.is_valid() {
            return Err(anyhow!("RCL context is not valid"));
        }
        
        unsafe {
            let mut allocator = rcutils_get_default_allocator();
            let mut topic_names_and_types = rcl_names_and_types_t { 
                names: rcutils_get_zero_initialized_string_array(),
                types: ptr::null_mut(),
            };
            
            let ret = rcl_get_topic_names_and_types(
                &self.node,
                &mut allocator as *mut _,
                false, // no_demangle
                &mut topic_names_and_types,
            );
            
            if ret != 0 {
                return Err(anyhow!("Failed to get topic names: {}", ret));
            }
            
            // Convert the topic names to Vec<String>
            let mut result = Vec::new();
            for i in 0..topic_names_and_types.names.size {
                if !topic_names_and_types.names.data.add(i).is_null() {
                    let name_ptr = *topic_names_and_types.names.data.add(i);
                    if !name_ptr.is_null() {
                        let name_cstr = std::ffi::CStr::from_ptr(name_ptr);
                        if let Ok(name_str) = name_cstr.to_str() {
                            result.push(name_str.to_string());
                        }
                    }
                }
            }
            
            // Clean up
            rcl_names_and_types_fini(&mut topic_names_and_types);
            
            Ok(result)
        }
    }

    /// Get all nodes in the ROS graph using direct RCL API calls
    #[allow(dead_code)]
    pub fn get_node_names(&self) -> Result<Vec<String>> {
        if !self.is_valid() {
            return Err(anyhow!("RCL context is not valid"));
        }
        
        unsafe {
            let allocator = rcutils_get_default_allocator();
            let mut node_names = rcutils_get_zero_initialized_string_array();
            let mut node_namespaces = rcutils_get_zero_initialized_string_array();
            
            let ret = rcl_get_node_names(
                &self.node,
                allocator,
                &mut node_names,
                &mut node_namespaces,
            );
            
            if ret != 0 {
                return Err(anyhow!("Failed to get node names: {}", ret));
            }
            
            // Convert the string array to Vec<String>
            let mut result = Vec::new();
            for i in 0..node_names.size {
                if !node_names.data.add(i).is_null() {
                    let name_ptr = *node_names.data.add(i);
                    if !name_ptr.is_null() {
                        let name_cstr = std::ffi::CStr::from_ptr(name_ptr);
                        if let Ok(name_str) = name_cstr.to_str() {
                            result.push(name_str.to_string());
                        }
                    }
                }
            }
            
            // Clean up
            rcutils_string_array_fini(&mut node_names);
            rcutils_string_array_fini(&mut node_namespaces);
            
            Ok(result)
        }
    }

    /// Get all services in the ROS graph using direct RCL API calls
    #[allow(dead_code)]
    pub fn get_service_names(&self) -> Result<Vec<String>> {
        if !self.is_valid() {
            return Err(anyhow!("RCL context is not valid"));
        }
        
        unsafe {
            let mut allocator = rcutils_get_default_allocator();
            let mut service_names_and_types = rcl_names_and_types_t { 
                names: rcutils_get_zero_initialized_string_array(),
                types: ptr::null_mut(),
            };
            
            let ret = rcl_get_service_names_and_types(
                &self.node,
                &mut allocator as *mut _,
                &mut service_names_and_types,
            );
            
            if ret != 0 {
                return Err(anyhow!("Failed to get service names: {}", ret));
            }
            
            // Convert the service names to Vec<String>
            let mut result = Vec::new();
            for i in 0..service_names_and_types.names.size {
                if !service_names_and_types.names.data.add(i).is_null() {
                    let name_ptr = *service_names_and_types.names.data.add(i);
                    if !name_ptr.is_null() {
                        let name_cstr = std::ffi::CStr::from_ptr(name_ptr);
                        if let Ok(name_str) = name_cstr.to_str() {
                            result.push(name_str.to_string());
                        }
                    }
                }
            }
            
            // Clean up
            rcl_names_and_types_fini(&mut service_names_and_types);
            
            Ok(result)
        }
    }

    /// Get all topics in the ROS graph with their type information using direct RCL API calls
    pub fn get_topics_with_types(&self) -> Result<Vec<TopicInfo>> {
        if !self.is_valid() {
            return Err(anyhow!("RCL context is not valid"));
        }
        
        unsafe {
            let mut allocator = rcutils_get_default_allocator();
            let mut topic_names_and_types = rcl_names_and_types_t { 
                names: rcutils_get_zero_initialized_string_array(),
                types: ptr::null_mut(),
            };
            
            let ret = rcl_get_topic_names_and_types(
                &self.node,
                &mut allocator,
                false, // no_demangle
                &mut topic_names_and_types,
            );
            
            if ret != 0 {
                return Err(anyhow!("Failed to get topic names and types: {}", ret));
            }
            
            // Convert the topic names and types to Vec<TopicInfo>
            let mut result = Vec::new();
            for i in 0..topic_names_and_types.names.size {
                if !topic_names_and_types.names.data.add(i).is_null() {
                    let name_ptr = *topic_names_and_types.names.data.add(i);
                    if !name_ptr.is_null() {
                        let name_cstr = std::ffi::CStr::from_ptr(name_ptr);
                        if let Ok(name_str) = name_cstr.to_str() {
                            // Get the types for this topic
                            let mut topic_types = Vec::new();
                            if !topic_names_and_types.types.is_null() {
                                let types_array = topic_names_and_types.types.add(i);
                                if !types_array.is_null() {
                                    let types_for_topic = &*types_array;
                                    for j in 0..types_for_topic.size {
                                        if !types_for_topic.data.add(j).is_null() {
                                            let type_ptr = *types_for_topic.data.add(j);
                                            if !type_ptr.is_null() {
                                                let type_cstr = std::ffi::CStr::from_ptr(type_ptr);
                                                if let Ok(type_str) = type_cstr.to_str() {
                                                    topic_types.push(type_str.to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            
                            result.push(TopicInfo {
                                name: name_str.to_string(),
                                types: topic_types,
                            });
                        }
                    }
                }
            }
            
            // Clean up
            rcl_names_and_types_fini(&mut topic_names_and_types);
            
            Ok(result)
        }
    }

    /// Get all topics and their types as tuples
    pub fn get_topic_names_and_types(&self) -> Result<Vec<(String, String)>> {
        if !self.is_valid() {
            return Err(anyhow!("RCL context is not valid"));
        }
        
        unsafe {
            let mut allocator = rcutils_get_default_allocator();
            let mut topic_names_and_types = rcl_names_and_types_t { 
                names: rcutils_get_zero_initialized_string_array(),
                types: ptr::null_mut(),
            };
            
            let ret = rcl_get_topic_names_and_types(
                &self.node,
                &mut allocator as *mut _,
                false, // no_demangle
                &mut topic_names_and_types,
            );
            
            if ret != 0 {
                return Err(anyhow!("Failed to get topic names and types: {}", ret));
            }
            
            // Convert the topics and types to Vec<(String, String)>
            let mut result = Vec::new();
            for i in 0..topic_names_and_types.names.size {
                if !topic_names_and_types.names.data.add(i).is_null() {
                    let name_ptr = *topic_names_and_types.names.data.add(i);
                    if !name_ptr.is_null() {
                        let name_cstr = std::ffi::CStr::from_ptr(name_ptr);
                        if let Ok(name_str) = name_cstr.to_str() {
                            // Get the corresponding type(s) - there may be multiple types per topic
                            if !topic_names_and_types.types.add(i).is_null() {
                                let types_array = &*topic_names_and_types.types.add(i);
                                for j in 0..types_array.size {
                                    if !types_array.data.add(j).is_null() {
                                        let type_ptr = *types_array.data.add(j);
                                        if !type_ptr.is_null() {
                                            let type_cstr = std::ffi::CStr::from_ptr(type_ptr);
                                            if let Ok(type_str) = type_cstr.to_str() {
                                                result.push((name_str.to_string(), type_str.to_string()));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Clean up
            rcl_names_and_types_fini(&mut topic_names_and_types);
            
            Ok(result)
        }
    }

    /// Count the number of publishers for a given topic
    pub fn count_publishers(&self, topic_name: &str) -> Result<usize> {
        if !self.is_valid() {
            return Err(anyhow!("RCL context is not valid"));
        }
        
        let topic_name_c = CString::new(topic_name).map_err(|e| anyhow!("Invalid topic name: {}", e))?;
        
        unsafe {
            let mut count: usize = 0;
            let ret = rcl_count_publishers(
                &self.node,
                topic_name_c.as_ptr(),
                &mut count,
            );
            
            if ret != 0 {
                return Err(anyhow!("Failed to count publishers for topic '{}': {}", topic_name, ret));
            }
            
            Ok(count)
        }
    }

    /// Count the number of subscribers for a given topic
    pub fn count_subscribers(&self, topic_name: &str) -> Result<usize> {
        if !self.is_valid() {
            return Err(anyhow!("RCL context is not valid"));
        }
        
        let topic_name_c = CString::new(topic_name).map_err(|e| anyhow!("Invalid topic name: {}", e))?;
        
        unsafe {
            let mut count: usize = 0;
            let ret = rcl_count_subscribers(
                &self.node,
                topic_name_c.as_ptr(),
                &mut count,
            );
            
            if ret != 0 {
                return Err(anyhow!("Failed to count subscribers for topic '{}': {}", topic_name, ret));
            }
            
            Ok(count)
        }
    }

    /// Get detailed information about all publishers to a topic
    pub fn get_publishers_info(&self, topic_name: &str) -> Result<Vec<TopicEndpointInfo>> {
        if !self.is_valid() {
            return Err(anyhow!("RCL context is not valid"));
        }
        
        let topic_name_c = CString::new(topic_name).map_err(|e| anyhow!("Invalid topic name: {}", e))?;
        
        unsafe {
            let mut allocator = rcutils_get_default_allocator();
            
            // Initialize the array - we need to use the RMW function since that's what RCL uses
            let mut publishers_info: rcl_topic_endpoint_info_array_t = std::mem::zeroed();
            
            let ret = rcl_get_publishers_info_by_topic(
                &self.node,
                &mut allocator,
                topic_name_c.as_ptr(),
                false, // no_mangle
                &mut publishers_info,
            );
            
            if ret != 0 {
                return Err(anyhow!("Failed to get publishers info for topic '{}': {}", topic_name, ret));
            }
            
            // Convert to our Rust struct
            let mut result = Vec::new();
            for i in 0..publishers_info.size {
                let info = &*(publishers_info.info_array.add(i));
                
                // Extract strings safely
                let node_name = if info.node_name.is_null() {
                    "unknown".to_string()
                } else {
                    std::ffi::CStr::from_ptr(info.node_name).to_string_lossy().to_string()
                };
                
                let node_namespace = if info.node_namespace.is_null() {
                    "/".to_string()
                } else {
                    std::ffi::CStr::from_ptr(info.node_namespace).to_string_lossy().to_string()
                };
                
                let topic_type = if info.topic_type.is_null() {
                    "unknown".to_string()
                } else {
                    std::ffi::CStr::from_ptr(info.topic_type).to_string_lossy().to_string()
                };
                
                // Extract topic type hash
                let topic_type_hash = format_topic_type_hash(&info.topic_type_hash);
                
                // Extract endpoint type
                let endpoint_type = EndpointType::from_rmw(info.endpoint_type);
                
                // Extract GID (Global ID) - it's a fixed-size array in RMW
                let gid = std::slice::from_raw_parts(info.endpoint_gid.as_ptr(), info.endpoint_gid.len()).to_vec();
                
                // Extract QoS profile
                let qos_profile = QosProfile::from_rmw(&info.qos_profile);
                
                result.push(TopicEndpointInfo {
                    node_name,
                    node_namespace,
                    topic_type,
                    topic_type_hash,
                    endpoint_type,
                    gid,
                    qos_profile,
                });
            }
            
            // We should clean up the array
            let mut allocator = rcutils_get_default_allocator();
            rmw_topic_endpoint_info_array_fini(&mut publishers_info, &mut allocator);
            
            Ok(result)
        }
    }

    /// Get detailed information about all subscribers to a topic
    pub fn get_subscribers_info(&self, topic_name: &str) -> Result<Vec<TopicEndpointInfo>> {
        if !self.is_valid() {
            return Err(anyhow!("RCL context is not valid"));
        }
        
        let topic_name_c = CString::new(topic_name).map_err(|e| anyhow!("Invalid topic name: {}", e))?;
        
        unsafe {
            let mut allocator = rcutils_get_default_allocator();
            
            // Initialize the array
            let mut subscribers_info: rcl_topic_endpoint_info_array_t = std::mem::zeroed();
            
            let ret = rcl_get_subscriptions_info_by_topic(
                &self.node,
                &mut allocator,
                topic_name_c.as_ptr(),
                false, // no_mangle
                &mut subscribers_info,
            );
            
            if ret != 0 {
                return Err(anyhow!("Failed to get subscribers info for topic '{}': {}", topic_name, ret));
            }
            
            // Convert to our Rust struct
            let mut result = Vec::new();
            for i in 0..subscribers_info.size {
                let info = &*(subscribers_info.info_array.add(i));
                
                // Extract strings safely
                let node_name = if info.node_name.is_null() {
                    "unknown".to_string()
                } else {
                    std::ffi::CStr::from_ptr(info.node_name).to_string_lossy().to_string()
                };
                
                let node_namespace = if info.node_namespace.is_null() {
                    "/".to_string()
                } else {
                    std::ffi::CStr::from_ptr(info.node_namespace).to_string_lossy().to_string()
                };
                
                let topic_type = if info.topic_type.is_null() {
                    "unknown".to_string()
                } else {
                    std::ffi::CStr::from_ptr(info.topic_type).to_string_lossy().to_string()
                };
                
                // Extract topic type hash
                let topic_type_hash = format_topic_type_hash(&info.topic_type_hash);
                
                // Extract endpoint type
                let endpoint_type = EndpointType::from_rmw(info.endpoint_type);
                
                // Extract GID (Global ID)
                let gid = std::slice::from_raw_parts(info.endpoint_gid.as_ptr(), info.endpoint_gid.len()).to_vec();
                
                // Extract QoS profile
                let qos_profile = QosProfile::from_rmw(&info.qos_profile);
                
                result.push(TopicEndpointInfo {
                    node_name,
                    node_namespace,
                    topic_type,
                    topic_type_hash,
                    endpoint_type,
                    gid,
                    qos_profile,
                });
            }

            // Clean up the array
            let mut allocator = rcutils_get_default_allocator();
            rmw_topic_endpoint_info_array_fini(&mut subscribers_info, &mut allocator);
            
            Ok(result)
        }
    }

    /// Check if a ROS 2 daemon is currently running
    pub fn is_daemon_running() -> bool {
        // Method 1: Try to check if ros2 daemon status command succeeds
        if let Ok(output) = Command::new("ros2")
            .args(&["daemon", "status"])
            .output()
        {
            // If the command succeeds and doesn't contain "not running", daemon is likely running
            let status_str = String::from_utf8_lossy(&output.stdout);
            return !status_str.contains("not running") && !status_str.contains("No daemon");
        }

        // Method 2: Check for typical daemon ports (fallback)
        // ROS 2 daemon typically uses port 11811 for the default domain
        let daemon_port = 11811 + env::var("ROS_DOMAIN_ID")
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(0);
        
        if let Ok(_) = TcpStream::connect(format!("127.0.0.1:{}", daemon_port)) {
            return true;
        }

        false
    }

    /// Get daemon status as a human-readable string
    pub fn get_daemon_status() -> String {
        if Self::is_daemon_running() {
            "Daemon running".to_string()
        } else {
            "No daemon running".to_string()
        }
    }
}

impl Drop for RclGraphContext {
    fn drop(&mut self) {
        if self.is_initialized {
            unsafe {
                // It's important to shut down the node before the context
                if rcl_node_is_valid(&self.node) {
                    rcl_node_fini(&mut self.node);
                }
                if rcl_context_is_valid(&self.context) {
                    rcl_shutdown(&mut self.context);
                }
            }
            self.is_initialized = false;
        }
    }
}

/// Information about a topic, including its name and types
#[derive(Debug, Clone)]
pub struct TopicInfo {
    pub name: String,
    pub types: Vec<String>,
}

impl EndpointType {
    fn from_rmw(endpoint_type: rmw_endpoint_type_t) -> Self {
        match endpoint_type {
            rmw_endpoint_type_e_RMW_ENDPOINT_PUBLISHER => EndpointType::Publisher,
            rmw_endpoint_type_e_RMW_ENDPOINT_SUBSCRIPTION => EndpointType::Subscription,
            _ => EndpointType::Invalid,
        }
    }
}

impl QosHistoryPolicy {
    fn from_rmw(history: rmw_qos_history_policy_e) -> Self {
        match history {
            rmw_qos_history_policy_e_RMW_QOS_POLICY_HISTORY_SYSTEM_DEFAULT => QosHistoryPolicy::SystemDefault,
            rmw_qos_history_policy_e_RMW_QOS_POLICY_HISTORY_KEEP_LAST => QosHistoryPolicy::KeepLast,
            rmw_qos_history_policy_e_RMW_QOS_POLICY_HISTORY_KEEP_ALL => QosHistoryPolicy::KeepAll,
            _ => QosHistoryPolicy::Unknown,
        }
    }
    
    pub fn to_string(&self) -> &'static str {
        match self {
            QosHistoryPolicy::SystemDefault => "SYSTEM_DEFAULT",
            QosHistoryPolicy::KeepLast => "KEEP_LAST",
            QosHistoryPolicy::KeepAll => "KEEP_ALL",
            QosHistoryPolicy::Unknown => "UNKNOWN",
        }
    }
}

impl QosReliabilityPolicy {
    fn from_rmw(reliability: rmw_qos_reliability_policy_e) -> Self {
        match reliability {
            rmw_qos_reliability_policy_e_RMW_QOS_POLICY_RELIABILITY_SYSTEM_DEFAULT => QosReliabilityPolicy::SystemDefault,
            rmw_qos_reliability_policy_e_RMW_QOS_POLICY_RELIABILITY_RELIABLE => QosReliabilityPolicy::Reliable,
            rmw_qos_reliability_policy_e_RMW_QOS_POLICY_RELIABILITY_BEST_EFFORT => QosReliabilityPolicy::BestEffort,
            rmw_qos_reliability_policy_e_RMW_QOS_POLICY_RELIABILITY_BEST_AVAILABLE => QosReliabilityPolicy::BestAvailable,
            _ => QosReliabilityPolicy::Unknown,
        }
    }
    
    pub fn to_string(&self) -> &'static str {
        match self {
            QosReliabilityPolicy::SystemDefault => "SYSTEM_DEFAULT",
            QosReliabilityPolicy::Reliable => "RELIABLE",
            QosReliabilityPolicy::BestEffort => "BEST_EFFORT",
            QosReliabilityPolicy::Unknown => "UNKNOWN",
            QosReliabilityPolicy::BestAvailable => "BEST_AVAILABLE",
        }
    }
}

impl QosDurabilityPolicy {
    fn from_rmw(durability: rmw_qos_durability_policy_e) -> Self {
        match durability {
            rmw_qos_durability_policy_e_RMW_QOS_POLICY_DURABILITY_SYSTEM_DEFAULT => QosDurabilityPolicy::SystemDefault,
            rmw_qos_durability_policy_e_RMW_QOS_POLICY_DURABILITY_TRANSIENT_LOCAL => QosDurabilityPolicy::TransientLocal,
            rmw_qos_durability_policy_e_RMW_QOS_POLICY_DURABILITY_VOLATILE => QosDurabilityPolicy::Volatile,
            rmw_qos_durability_policy_e_RMW_QOS_POLICY_DURABILITY_BEST_AVAILABLE => QosDurabilityPolicy::BestAvailable,
            _ => QosDurabilityPolicy::Unknown,
        }
    }
    
    pub fn to_string(&self) -> &'static str {
        match self {
            QosDurabilityPolicy::SystemDefault => "SYSTEM_DEFAULT",
            QosDurabilityPolicy::TransientLocal => "TRANSIENT_LOCAL",
            QosDurabilityPolicy::Volatile => "VOLATILE",
            QosDurabilityPolicy::Unknown => "UNKNOWN",
            QosDurabilityPolicy::BestAvailable => "BEST_AVAILABLE",
        }
    }
}

impl QosLivelinessPolicy {
    fn from_rmw(liveliness: rmw_qos_liveliness_policy_e) -> Self {
        match liveliness {
            rmw_qos_liveliness_policy_e_RMW_QOS_POLICY_LIVELINESS_SYSTEM_DEFAULT => QosLivelinessPolicy::SystemDefault,
            rmw_qos_liveliness_policy_e_RMW_QOS_POLICY_LIVELINESS_AUTOMATIC => QosLivelinessPolicy::Automatic,
            rmw_qos_liveliness_policy_e_RMW_QOS_POLICY_LIVELINESS_MANUAL_BY_TOPIC => QosLivelinessPolicy::ManualByTopic,
            rmw_qos_liveliness_policy_e_RMW_QOS_POLICY_LIVELINESS_BEST_AVAILABLE => QosLivelinessPolicy::BestAvailable,
            _ => QosLivelinessPolicy::Unknown,
        }
    }
    
    pub fn to_string(&self) -> &'static str {
        match self {
            QosLivelinessPolicy::SystemDefault => "SYSTEM_DEFAULT",
            QosLivelinessPolicy::Automatic => "AUTOMATIC",
            QosLivelinessPolicy::ManualByNode => "MANUAL_BY_NODE",
            QosLivelinessPolicy::ManualByTopic => "MANUAL_BY_TOPIC",
            QosLivelinessPolicy::Unknown => "UNKNOWN",
            QosLivelinessPolicy::BestAvailable => "BEST_AVAILABLE",
        }
    }
}

impl QosProfile {
    fn from_rmw(qos: &rmw_qos_profile_t) -> Self {
        QosProfile {
            history: QosHistoryPolicy::from_rmw(qos.history),
            depth: qos.depth,
            reliability: QosReliabilityPolicy::from_rmw(qos.reliability),
            durability: QosDurabilityPolicy::from_rmw(qos.durability),
            deadline_sec: qos.deadline.sec,
            deadline_nsec: qos.deadline.nsec,
            lifespan_sec: qos.lifespan.sec,
            lifespan_nsec: qos.lifespan.nsec,
            liveliness: QosLivelinessPolicy::from_rmw(qos.liveliness),
            liveliness_lease_duration_sec: qos.liveliness_lease_duration.sec,
            liveliness_lease_duration_nsec: qos.liveliness_lease_duration.nsec,
            avoid_ros_namespace_conventions: qos.avoid_ros_namespace_conventions,
        }
    }
    
    pub fn format_duration(&self, sec: u64, nsec: u64) -> String {
        if sec == 0x7FFFFFFFFFFFFFFF && nsec == 0x7FFFFFFFFFFFFFFF {
            "infinite".to_string()
        } else if sec == 0 && nsec == 0 {
            "0.000000000".to_string()
        } else {
            format!("{}.{:09}", sec, nsec)
        }
    }
}

// Helper function to format topic type hash
fn format_topic_type_hash(hash: &rosidl_type_hash_t) -> String {
    // Format the hash as a hexadecimal string
    let hash_bytes = unsafe {
        std::slice::from_raw_parts(hash.value.as_ptr(), hash.value.len())
    };
    hash_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>()
}

// Helper function to format GID
#[allow(dead_code)]
fn format_gid(gid: &[u8]) -> String {
    gid.iter().map(|b| format!("{:02x}", b)).collect::<Vec<String>>().join(".")
}
