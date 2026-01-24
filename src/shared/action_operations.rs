use anyhow::{anyhow, Result};

use crate::shared::graph_context::RclGraphContext;

/// Discover action servers by scanning the ROS graph for action service endpoints.
///
/// ROS 2 actions are implemented using services with these names:
/// - `<action_name>/_action/send_goal`
/// - `<action_name>/_action/cancel_goal`
/// - `<action_name>/_action/get_result`
/// plus topics for feedback/status.
///
/// `rclrs` doesn't currently expose an action graph API, so we infer action names
/// from the service list.
pub fn get_action_names(context: &RclGraphContext) -> Result<Vec<String>> {
    if !context.is_valid() {
        return Err(anyhow!("RCL context is not valid"));
    }

    let map = context.node().get_service_names_and_types()?;
    let mut names = Vec::new();
    for svc_name in map.keys() {
        if let Some(action_name) = svc_name.strip_suffix("/_action/send_goal") {
            names.push(action_name.to_string());
        }
    }
    names.sort();
    names.dedup();
    Ok(names)
}

/// Best-effort: infer the action type from the `send_goal` service type.
///
/// The `send_goal` service type is `<pkg>/action/<Action>_SendGoal`.
/// From that we infer `<pkg>/action/<Action>`.
pub fn get_action_type(context: &RclGraphContext, action_name: &str) -> Result<Option<String>> {
    if !context.is_valid() {
        return Err(anyhow!("RCL context is not valid"));
    }

    let svc_name = format!("{}/_action/send_goal", action_name);
    let map = context.node().get_service_names_and_types()?;
    let Some(types) = map.get(&svc_name) else {
        return Ok(None);
    };

    for ty in types {
        if let Some((pkg, rest)) = ty.split_once("/action/") {
            if let Some(base) = rest.strip_suffix("_SendGoal") {
                return Ok(Some(format!("{}/action/{}", pkg, base)));
            }
        }
    }

    Ok(None)
}
