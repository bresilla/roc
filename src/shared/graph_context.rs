#![allow(non_upper_case_globals)]

use rclrs::*;
use std::ptr;
use std::ffi::CString;
use std::env;
use std::process::Command;
use std::net::TcpStream;
use std::time::Duration;
use anyhow::{Result, anyhow};

/// A simple RCL context manager for graph operations  
/// Includes both context and a minimal node for graph queries
pub struct RclGraphContext {
    context: rcl_context_t,
    node: rcl_node_t,
    is_initialized: bool,
}

impl RclGraphContext {
    /// Create a new RCL context for graph operations
    /// Note: This implementation always performs direct DDS discovery (equivalent to --no-daemon)
    pub fn new() -> Result<Self> {
        Self::new_with_discovery(std::time::Duration::from_millis(300))
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

    /// Get a reference to the underlying node for RCL operations
    pub fn node(&self) -> &rcl_node_t {
        &self.node
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

    /// Wait for a specific topic to appear in the graph
    /// This is useful when subscribing to topics that might not be discovered yet
    pub fn wait_for_topic(&self, topic_name: &str, timeout: Duration) -> Result<bool> {
        if !self.is_valid() {
            return Err(anyhow!("RCL context is not valid"));
        }

        let start_time = std::time::Instant::now();
        let check_interval = Duration::from_millis(50);

        while start_time.elapsed() < timeout {
            if let Ok(topics) = crate::shared::topic_operations::get_topic_names(self) {
                if topics.contains(&topic_name.to_string()) {
                    return Ok(true);
                }
            }
            std::thread::sleep(check_interval);
        }

        Ok(false)
    }

    /// Wait for a specific topic with publishers to appear
    /// This ensures not just that the topic exists, but that it has active publishers
    pub fn wait_for_topic_with_publishers(&self, topic_name: &str, timeout: Duration) -> Result<bool> {
        if !self.is_valid() {
            return Err(anyhow!("RCL context is not valid"));
        }

        let start_time = std::time::Instant::now();
        let check_interval = Duration::from_millis(100);

        while start_time.elapsed() < timeout {
            if let Ok(pub_count) = crate::shared::topic_operations::count_publishers(self, topic_name) {
                if pub_count > 0 {
                    return Ok(true);
                }
            }
            std::thread::sleep(check_interval);
        }

        Ok(false)
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