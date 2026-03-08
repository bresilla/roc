use anyhow::{anyhow, Result};
use crate::shared::graph_context::{parse_discovery_duration, DEFAULT_DISCOVERY_TIME};
use crate::shared::ros_names::is_hidden_node_name;
use rclrs::{
    Client, Context, CreateBasicExecutor, Executor, IntoNodeOptions, Node, RclrsErrorFilter,
    SpinOptions,
};
use regex::Regex;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rclrs::vendor::rcl_interfaces::{
    msg::{Parameter, ParameterType, ParameterValue},
    srv::{
        DescribeParameters, DescribeParameters_Request, DescribeParameters_Response,
        GetParameterTypes, GetParameterTypes_Request, GetParameterTypes_Response, GetParameters,
        GetParameters_Request, GetParameters_Response, ListParameters, ListParameters_Request,
        ListParameters_Response, SetParameters, SetParameters_Request, SetParameters_Response,
    },
};

const DEFAULT_SERVICE_READY_TIMEOUT: Duration = Duration::from_secs(8);
const DEFAULT_CALL_TIMEOUT: Duration = Duration::from_secs(8);

/// A small helper for calling parameter services on a remote node.
///
/// This uses the standard ROS 2 parameter service endpoints that every node
/// provides when it has parameter services enabled.
pub struct ParamClientContext {
    _context: Context,
    pub executor: Executor,
    pub node: Node,
}

impl ParamClientContext {
    pub fn new() -> Result<Self> {
        Self::new_with_discovery_options(DEFAULT_DISCOVERY_TIME, false)
    }

    pub fn new_with_options(spin_time: Option<&str>, use_sim_time: bool) -> Result<Self> {
        Self::new_with_discovery_options(parse_discovery_duration(spin_time)?, use_sim_time)
    }

    pub fn new_with_discovery_options(discovery_time: Duration, use_sim_time: bool) -> Result<Self> {
        let context = Context::default_from_env()?;
        let executor = context.create_basic_executor();
        let node = executor.create_node("roc_param_cli".arguments(cli_node_arguments(use_sim_time)))?;

        // Give DDS a short discovery window so parameter service servers
        // show up before we try to wait on readiness.
        std::thread::sleep(discovery_time);
        Ok(Self {
            _context: context,
            executor,
            node,
        })
    }

    pub fn node_fqn(user_input: &str) -> String {
        let trimmed = user_input.trim();
        if trimmed.starts_with('/') {
            trimmed.to_string()
        } else {
            format!("/{trimmed}")
        }
    }

    pub fn ensure_node_available(
        &self,
        node_fqn: &str,
        include_hidden_nodes: bool,
    ) -> Result<()> {
        let nodes = self.node.get_node_names()?;
        let found = nodes.into_iter().any(|node| {
            let full_name = if node.namespace == "/" {
                format!("/{}", node.name)
            } else if node.namespace.ends_with('/') {
                format!("{}{}", node.namespace, node.name)
            } else {
                format!("{}/{}", node.namespace, node.name)
            };

            if !include_hidden_nodes && is_hidden_node_name(&full_name) {
                return false;
            }

            full_name == node_fqn
        });

        if found {
            Ok(())
        } else {
            Err(anyhow!("Node '{}' not found", node_fqn))
        }
    }

    pub fn list_parameters_client(&self, node_fqn: &str) -> Result<Client<ListParameters>> {
        let service_name = format!("{node_fqn}/list_parameters");
        Ok(self.node.create_client::<ListParameters>(&service_name)?)
    }

    pub fn get_parameters_client(&self, node_fqn: &str) -> Result<Client<GetParameters>> {
        let service_name = format!("{node_fqn}/get_parameters");
        Ok(self.node.create_client::<GetParameters>(&service_name)?)
    }

    pub fn get_parameter_types_client(&self, node_fqn: &str) -> Result<Client<GetParameterTypes>> {
        let service_name = format!("{node_fqn}/get_parameter_types");
        Ok(self
            .node
            .create_client::<GetParameterTypes>(&service_name)?)
    }

    pub fn describe_parameters_client(&self, node_fqn: &str) -> Result<Client<DescribeParameters>> {
        let service_name = format!("{node_fqn}/describe_parameters");
        Ok(self
            .node
            .create_client::<DescribeParameters>(&service_name)?)
    }

    pub fn set_parameters_client(&self, node_fqn: &str) -> Result<Client<SetParameters>> {
        let service_name = format!("{node_fqn}/set_parameters");
        Ok(self.node.create_client::<SetParameters>(&service_name)?)
    }

    pub fn wait_for_service_ready<T: rclrs::ServiceIDL>(
        &mut self,
        client: &Client<T>,
        timeout: Duration,
    ) -> Result<()> {
        // NOTE: Waiting on `notify_on_service_ready()` can hang if the server is
        // already present in the ROS graph but no new graph-change event occurs.
        // A simple poll loop is more reliable across DDS vendors.
        let start = std::time::Instant::now();
        let mut last_err: Option<anyhow::Error> = None;

        while start.elapsed() < timeout {
            match client.service_is_ready() {
                Ok(true) => return Ok(()),
                Ok(false) => {}
                Err(e) => last_err = Some(anyhow!(e)),
            }

            // Give the executor a short chance to process any incoming graph
            // events and then sleep briefly.
            let _ = self
                .executor
                .spin(SpinOptions::default().timeout(Duration::from_millis(50)));
            std::thread::sleep(Duration::from_millis(50));
        }

        if let Some(e) = last_err {
            return Err(anyhow!(
                "Service '{}' not ready: {}",
                client.service_name(),
                e
            ));
        }

        Err(anyhow!(
            "Service '{}' not ready: timeout",
            client.service_name()
        ))
    }

    pub fn call_and_capture<T: rclrs::ServiceIDL>(
        &mut self,
        client: &Client<T>,
        request: &T::Request,
        timeout: Duration,
    ) -> Result<T::Response> {
        let captured: Arc<Mutex<Option<T::Response>>> = Arc::new(Mutex::new(None));
        let captured_inner = Arc::clone(&captured);
        let promise = client.call_then(request, move |resp: T::Response| {
            if let Ok(mut captured) = captured_inner.lock() {
                *captured = Some(resp);
            }
        })?;

        self.executor
            .spin(
                SpinOptions::default()
                    .until_promise_resolved(promise)
                    .timeout(timeout),
            )
            .first_error()?;

        let result = captured
            .lock()
            .map_err(|_| anyhow!("Service response capture state poisoned"))?
            .clone();
        result.ok_or_else(|| {
            anyhow!(
                "No response captured from service '{}'",
                client.service_name()
            )
        })
    }

    pub fn list_parameters(
        &mut self,
        node_fqn: &str,
        prefixes: Vec<String>,
    ) -> Result<ListParameters_Response> {
        let client = self.list_parameters_client(node_fqn)?;
        self.wait_for_service_ready(&client, DEFAULT_SERVICE_READY_TIMEOUT)?;
        let request = ListParameters_Request {
            prefixes,
            depth: ListParameters_Request::DEPTH_RECURSIVE,
        };
        self.call_and_capture(&client, &request, DEFAULT_CALL_TIMEOUT)
    }

    pub fn get_parameters(
        &mut self,
        node_fqn: &str,
        names: Vec<String>,
    ) -> Result<GetParameters_Response> {
        let client = self.get_parameters_client(node_fqn)?;
        self.wait_for_service_ready(&client, DEFAULT_SERVICE_READY_TIMEOUT)?;
        let request = GetParameters_Request { names };
        self.call_and_capture(&client, &request, DEFAULT_CALL_TIMEOUT)
    }

    pub fn get_parameter_types(
        &mut self,
        node_fqn: &str,
        names: Vec<String>,
    ) -> Result<GetParameterTypes_Response> {
        let client = self.get_parameter_types_client(node_fqn)?;
        self.wait_for_service_ready(&client, DEFAULT_SERVICE_READY_TIMEOUT)?;
        let request = GetParameterTypes_Request { names };
        self.call_and_capture(&client, &request, DEFAULT_CALL_TIMEOUT)
    }

    pub fn describe_parameters(
        &mut self,
        node_fqn: &str,
        names: Vec<String>,
    ) -> Result<DescribeParameters_Response> {
        let client = self.describe_parameters_client(node_fqn)?;
        self.wait_for_service_ready(&client, DEFAULT_SERVICE_READY_TIMEOUT)?;
        let request = DescribeParameters_Request { names };
        self.call_and_capture(&client, &request, DEFAULT_CALL_TIMEOUT)
    }

    pub fn set_parameters(
        &mut self,
        node_fqn: &str,
        parameters: Vec<Parameter>,
    ) -> Result<SetParameters_Response> {
        let client = self.set_parameters_client(node_fqn)?;
        self.wait_for_service_ready(&client, DEFAULT_SERVICE_READY_TIMEOUT)?;
        let request = SetParameters_Request { parameters };
        self.call_and_capture(&client, &request, DEFAULT_CALL_TIMEOUT)
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

pub fn parameter_type_to_string(t: u8) -> &'static str {
    match t {
        ParameterType::PARAMETER_NOT_SET => "not set",
        ParameterType::PARAMETER_BOOL => "bool",
        ParameterType::PARAMETER_INTEGER => "integer",
        ParameterType::PARAMETER_DOUBLE => "double",
        ParameterType::PARAMETER_STRING => "string",
        ParameterType::PARAMETER_BYTE_ARRAY => "byte_array",
        ParameterType::PARAMETER_BOOL_ARRAY => "bool_array",
        ParameterType::PARAMETER_INTEGER_ARRAY => "integer_array",
        ParameterType::PARAMETER_DOUBLE_ARRAY => "double_array",
        ParameterType::PARAMETER_STRING_ARRAY => "string_array",
        _ => "unknown",
    }
}

pub fn format_parameter_value_for_display(value: &ParameterValue, hide_type: bool) -> String {
    if value.type_ == ParameterType::PARAMETER_NOT_SET {
        return "Parameter not set".to_string();
    }

    let render_scalar = || match value.type_ {
        ParameterType::PARAMETER_BOOL => value.bool_value.to_string(),
        ParameterType::PARAMETER_INTEGER => value.integer_value.to_string(),
        ParameterType::PARAMETER_DOUBLE => value.double_value.to_string(),
        ParameterType::PARAMETER_STRING => value.string_value.clone(),
        _ => "".to_string(),
    };

    let render_array = || match value.type_ {
        ParameterType::PARAMETER_BYTE_ARRAY => format!(
            "[{}]",
            value
                .byte_array_value
                .iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        ParameterType::PARAMETER_BOOL_ARRAY => format!(
            "[{}]",
            value
                .bool_array_value
                .iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        ParameterType::PARAMETER_INTEGER_ARRAY => format!(
            "[{}]",
            value
                .integer_array_value
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        ParameterType::PARAMETER_DOUBLE_ARRAY => format!(
            "[{}]",
            value
                .double_array_value
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        ParameterType::PARAMETER_STRING_ARRAY => format!(
            "[{}]",
            value
                .string_array_value
                .iter()
                .map(|s| format!("\"{}\"", s))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        _ => "".to_string(),
    };

    if hide_type {
        return match value.type_ {
            ParameterType::PARAMETER_BOOL
            | ParameterType::PARAMETER_INTEGER
            | ParameterType::PARAMETER_DOUBLE
            | ParameterType::PARAMETER_STRING => render_scalar(),
            ParameterType::PARAMETER_BYTE_ARRAY
            | ParameterType::PARAMETER_BOOL_ARRAY
            | ParameterType::PARAMETER_INTEGER_ARRAY
            | ParameterType::PARAMETER_DOUBLE_ARRAY
            | ParameterType::PARAMETER_STRING_ARRAY => render_array(),
            _ => "<unsupported>".to_string(),
        };
    }

    match value.type_ {
        ParameterType::PARAMETER_BOOL => {
            format!(
                "Boolean value is: {}",
                if value.bool_value { "True" } else { "False" }
            )
        }
        ParameterType::PARAMETER_INTEGER => format!("Integer value is: {}", value.integer_value),
        ParameterType::PARAMETER_DOUBLE => format!("Double value is: {}", value.double_value),
        ParameterType::PARAMETER_STRING => format!("String value is: {}", value.string_value),
        ParameterType::PARAMETER_BYTE_ARRAY => format!("Byte array value is: {}", render_array()),
        ParameterType::PARAMETER_BOOL_ARRAY => {
            format!("Boolean array value is: {}", render_array())
        }
        ParameterType::PARAMETER_INTEGER_ARRAY => {
            format!("Integer array value is: {}", render_array())
        }
        ParameterType::PARAMETER_DOUBLE_ARRAY => {
            format!("Double array value is: {}", render_array())
        }
        ParameterType::PARAMETER_STRING_ARRAY => {
            format!("String array value is: {}", render_array())
        }
        _ => "<unsupported>".to_string(),
    }
}

pub fn parse_value_tokens_to_parameter_value(value_tokens: &[String]) -> Result<ParameterValue> {
    let joined = value_tokens
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    let s = joined.trim();

    let mut v = ParameterValue::default();

    // Arrays: [a, b, c]
    if s.starts_with('[') && s.ends_with(']') {
        let inner = s[1..s.len() - 1].trim();
        if inner.is_empty() {
            // Default to empty string array (matches ros2 accepting "[]")
            v.type_ = ParameterType::PARAMETER_STRING_ARRAY;
            v.string_array_value = Vec::new();
            return Ok(v);
        }

        let parts: Vec<String> = inner.split(',').map(|p| p.trim().to_string()).collect();

        // Try bool array
        if let Some(parsed) = try_parse_bool_array(&parts) {
            v.type_ = ParameterType::PARAMETER_BOOL_ARRAY;
            v.bool_array_value = parsed;
            return Ok(v);
        }
        // Try int array
        if let Some(parsed) = try_parse_i64_array(&parts) {
            v.type_ = ParameterType::PARAMETER_INTEGER_ARRAY;
            v.integer_array_value = parsed;
            return Ok(v);
        }
        // Try float array
        if let Some(parsed) = try_parse_f64_array(&parts) {
            v.type_ = ParameterType::PARAMETER_DOUBLE_ARRAY;
            v.double_array_value = parsed;
            return Ok(v);
        }

        // Fall back to string array
        v.type_ = ParameterType::PARAMETER_STRING_ARRAY;
        v.string_array_value = parts
            .into_iter()
            .map(|p| strip_quotes(&p).to_string())
            .collect();
        return Ok(v);
    }

    // Scalars
    if let Some(b) = parse_bool(s) {
        v.type_ = ParameterType::PARAMETER_BOOL;
        v.bool_value = b;
        return Ok(v);
    }
    if let Ok(i) = s.parse::<i64>() {
        v.type_ = ParameterType::PARAMETER_INTEGER;
        v.integer_value = i;
        return Ok(v);
    }
    if let Ok(f) = s.parse::<f64>() {
        v.type_ = ParameterType::PARAMETER_DOUBLE;
        v.double_value = f;
        return Ok(v);
    }

    v.type_ = ParameterType::PARAMETER_STRING;
    v.string_value = strip_quotes(s).to_string();
    Ok(v)
}

fn parse_bool(s: &str) -> Option<bool> {
    match s.to_ascii_lowercase().as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn try_parse_bool_array(parts: &[String]) -> Option<Vec<bool>> {
    let mut out = Vec::with_capacity(parts.len());
    for p in parts {
        out.push(parse_bool(p.trim())?);
    }
    Some(out)
}

fn try_parse_i64_array(parts: &[String]) -> Option<Vec<i64>> {
    let mut out = Vec::with_capacity(parts.len());
    for p in parts {
        out.push(p.trim().parse::<i64>().ok()?);
    }
    Some(out)
}

fn try_parse_f64_array(parts: &[String]) -> Option<Vec<f64>> {
    let mut out = Vec::with_capacity(parts.len());
    for p in parts {
        out.push(p.trim().parse::<f64>().ok()?);
    }
    Some(out)
}

fn strip_quotes(s: &str) -> &str {
    let s = s.trim();
    if s.len() >= 2 {
        let bytes = s.as_bytes();
        let first = bytes[0] as char;
        let last = bytes[bytes.len() - 1] as char;
        if (first == '"' && last == '"') || (first == '\'' && last == '\'') {
            return &s[1..s.len() - 1];
        }
    }
    s
}

pub fn filter_parameter_names(names: Vec<String>, filter: Option<&str>) -> Result<Vec<String>> {
    let Some(filter) = filter else {
        return Ok(names);
    };
    let re = Regex::new(filter).map_err(|e| anyhow!("Invalid regex '{filter}': {e}"))?;
    Ok(names.into_iter().filter(|n| re.is_match(n)).collect())
}

#[cfg(test)]
mod tests {
    use super::cli_node_arguments;

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
