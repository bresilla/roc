use anyhow::{anyhow, Result};
use rclrs::{Context, CreateBasicExecutor, Executor, IntoNodeOptions, Node};
use std::net::TcpStream;
use std::sync::Mutex;
use std::time::Duration;

pub const DEFAULT_DISCOVERY_TIME: Duration = Duration::from_millis(300);

/// A small, safe wrapper around an `rclrs` context + node.
///
/// This is used for ROS graph queries (topics, services, nodes) and lightweight
/// discovery polling.
pub struct RclGraphContext {
    context: Context,
    node: Node,
    executor: Mutex<Option<Executor>>,
}

impl RclGraphContext {
    /// Create a new graph context.
    pub fn new() -> Result<Self> {
        Self::new_with_discovery_options(DEFAULT_DISCOVERY_TIME, false)
    }

    /// Create a new graph context.
    #[allow(dead_code)]
    pub fn new_no_daemon() -> Result<Self> {
        // This tool always does direct DDS discovery.
        Self::new()
    }

    /// Create a new graph context honoring ROS-style CLI discovery options.
    pub fn new_with_options(spin_time: Option<&str>, use_sim_time: bool) -> Result<Self> {
        Self::new_with_discovery_options(parse_discovery_duration(spin_time)?, use_sim_time)
    }

    /// Create a new graph context with explicit discovery and sim-time options.
    pub fn new_with_discovery_options(
        discovery_time: Duration,
        use_sim_time: bool,
    ) -> Result<Self> {
        let context = Context::default_from_env()?;
        let executor = context.create_basic_executor();
        let node =
            executor.create_node("roc_graph_node".arguments(cli_node_arguments(use_sim_time)))?;

        // Give DDS time to discover peers/topics.
        std::thread::sleep(discovery_time);

        Ok(Self {
            context,
            node,
            executor: Mutex::new(Some(executor)),
        })
    }

    /// Check if the context is valid.
    pub fn is_valid(&self) -> bool {
        self.context.ok()
    }

    /// Get the `rclrs` node used for graph queries.
    pub fn node(&self) -> &Node {
        &self.node
    }

    pub(crate) fn take_executor(&self) -> Result<Executor> {
        self.executor
            .lock()
            .map_err(|_| anyhow!("RCL executor state poisoned"))?
            .take()
            .ok_or_else(|| anyhow!("RCL executor is already attached to an active subscription"))
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

fn cli_node_arguments(use_sim_time: bool) -> Vec<String> {
    if use_sim_time {
        vec![
            "--ros-args".to_string(),
            "-p".to_string(),
            "use_sim_time:=true".to_string(),
        ]
    } else {
        Vec::new()
    }
}

pub(crate) fn parse_discovery_duration(spin_time: Option<&str>) -> Result<Duration> {
    let Some(raw_value) = spin_time.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(DEFAULT_DISCOVERY_TIME);
    };

    let (numeric, scale) = if let Some(value) = raw_value.strip_suffix("ms") {
        (value.trim(), 0.001)
    } else if let Some(value) = raw_value.strip_suffix('s') {
        (value.trim(), 1.0)
    } else {
        (raw_value, 1.0)
    };

    let seconds = numeric
        .parse::<f64>()
        .map_err(|_| anyhow!("Invalid --spin-time value '{}'", raw_value))?
        * scale;

    if !seconds.is_finite() || seconds < 0.0 {
        return Err(anyhow!("Invalid --spin-time value '{}'", raw_value));
    }

    Ok(Duration::from_secs_f64(seconds))
}

#[cfg(test)]
mod tests {
    use super::{cli_node_arguments, parse_discovery_duration, DEFAULT_DISCOVERY_TIME};
    use std::time::Duration;

    #[test]
    fn parse_discovery_duration_defaults_when_missing() {
        assert_eq!(
            parse_discovery_duration(None).unwrap(),
            DEFAULT_DISCOVERY_TIME
        );
        assert_eq!(
            parse_discovery_duration(Some("")).unwrap(),
            DEFAULT_DISCOVERY_TIME
        );
    }

    #[test]
    fn parse_discovery_duration_accepts_seconds() {
        assert_eq!(
            parse_discovery_duration(Some("0.5")).unwrap(),
            Duration::from_millis(500)
        );
        assert_eq!(
            parse_discovery_duration(Some("2s")).unwrap(),
            Duration::from_secs(2)
        );
    }

    #[test]
    fn parse_discovery_duration_accepts_milliseconds() {
        assert_eq!(
            parse_discovery_duration(Some("750ms")).unwrap(),
            Duration::from_millis(750)
        );
    }

    #[test]
    fn parse_discovery_duration_rejects_invalid_values() {
        assert!(parse_discovery_duration(Some("-1")).is_err());
        assert!(parse_discovery_duration(Some("abc")).is_err());
    }

    #[test]
    fn cli_node_arguments_enable_sim_time_when_requested() {
        assert_eq!(
            cli_node_arguments(true),
            vec!["--ros-args", "-p", "use_sim_time:=true"]
        );
    }

    #[test]
    fn cli_node_arguments_are_empty_by_default() {
        assert!(cli_node_arguments(false).is_empty());
    }
}
