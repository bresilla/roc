use anyhow::{anyhow, Result};

use crate::shared::graph_context::RclGraphContext;

/// Get all nodes in the ROS graph.
#[allow(dead_code)]
pub fn get_node_names(context: &RclGraphContext) -> Result<Vec<String>> {
    if !context.is_valid() {
        return Err(anyhow!("RCL context is not valid"));
    }

    let nodes = context.node().get_node_names()?;
    let mut names: Vec<String> = nodes.into_iter().map(|n| n.name).collect();
    names.sort();
    Ok(names)
}

/// Get all nodes with their namespaces in the ROS graph.
#[allow(dead_code)]
pub fn get_node_names_with_namespaces(context: &RclGraphContext) -> Result<Vec<(String, String)>> {
    if !context.is_valid() {
        return Err(anyhow!("RCL context is not valid"));
    }

    let nodes = context.node().get_node_names()?;
    let mut pairs: Vec<(String, String)> =
        nodes.into_iter().map(|n| (n.name, n.namespace)).collect();
    pairs.sort_by(|(a, an), (b, bn)| a.cmp(b).then(an.cmp(bn)));
    Ok(pairs)
}
