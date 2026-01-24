use anyhow::{anyhow, Result};
use rclrs::{Context, CreateBasicExecutor, Node};
use std::net::TcpStream;
use std::time::Duration;

/// A small, safe wrapper around an `rclrs` context + node.
///
/// This is used for ROS graph queries (topics, services, nodes) and lightweight
/// discovery polling.
pub struct RclGraphContext {
    context: Context,
    node: Node,
}

impl RclGraphContext {
    /// Create a new graph context.
    pub fn new() -> Result<Self> {
        Self::new_with_discovery(Duration::from_millis(300))
    }

    /// Create a new graph context.
    #[allow(dead_code)]
    pub fn new_no_daemon() -> Result<Self> {
        // This tool always does direct DDS discovery.
        Self::new()
    }

    /// Create a new graph context and wait for a short discovery window.
    pub fn new_with_discovery(discovery_time: Duration) -> Result<Self> {
        let context = Context::default_from_env()?;
        let executor = context.create_basic_executor();
        let node = executor.create_node("roc_graph_node")?;

        // Give DDS time to discover peers/topics.
        std::thread::sleep(discovery_time);

        Ok(Self { context, node })
    }

    /// Check if the context is valid.
    pub fn is_valid(&self) -> bool {
        self.context.ok()
    }

    /// Get the `rclrs` node used for graph queries.
    pub fn node(&self) -> &Node {
        &self.node
    }

    /// Wait for a topic to appear in the graph.
    pub fn wait_for_topic(&self, topic_name: &str, timeout: Duration) -> Result<bool> {
        if !self.is_valid() {
            return Err(anyhow!("RCL context is not valid"));
        }

        let start = std::time::Instant::now();
        let interval = Duration::from_millis(50);
        while start.elapsed() < timeout {
            let topics = crate::shared::topic_operations::get_topic_names(self)?;
            if topics.iter().any(|t| t == topic_name) {
                return Ok(true);
            }
            std::thread::sleep(interval);
        }
        Ok(false)
    }

    /// Wait for a topic to have at least one publisher.
    pub fn wait_for_topic_with_publishers(
        &self,
        topic_name: &str,
        timeout: Duration,
    ) -> Result<bool> {
        if !self.is_valid() {
            return Err(anyhow!("RCL context is not valid"));
        }

        let start = std::time::Instant::now();
        let interval = Duration::from_millis(100);
        while start.elapsed() < timeout {
            if crate::shared::topic_operations::count_publishers(self, topic_name)? > 0 {
                return Ok(true);
            }
            std::thread::sleep(interval);
        }
        Ok(false)
    }

    /// Check if a ROS 2 daemon is currently running.
    pub fn is_daemon_running() -> bool {
        let daemon_port = 11811
            + std::env::var("ROS_DOMAIN_ID")
                .ok()
                .and_then(|s| s.parse::<u16>().ok())
                .unwrap_or(0);

        TcpStream::connect(format!("127.0.0.1:{}", daemon_port)).is_ok()
    }

    /// Get daemon status as a human-readable string.
    pub fn get_daemon_status() -> String {
        if Self::is_daemon_running() {
            "Daemon running".to_string()
        } else {
            "No daemon running".to_string()
        }
    }
}
