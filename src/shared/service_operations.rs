use anyhow::{anyhow, Result};

use crate::shared::graph_context::RclGraphContext;

/// Get all services in the ROS graph.
#[allow(dead_code)]
pub fn get_service_names(context: &RclGraphContext) -> Result<Vec<String>> {
    if !context.is_valid() {
        return Err(anyhow!("RCL context is not valid"));
    }

    let map = context.node().get_service_names_and_types()?;
    let mut names: Vec<String> = map.keys().cloned().collect();
    names.sort();
    Ok(names)
}

/// Get all services and their types as tuples.
#[allow(dead_code)]
pub fn get_service_names_and_types(context: &RclGraphContext) -> Result<Vec<(String, String)>> {
    if !context.is_valid() {
        return Err(anyhow!("RCL context is not valid"));
    }

    let map = context.node().get_service_names_and_types()?;
    let mut pairs = Vec::new();
    for (name, types) in map {
        for ty in types {
            pairs.push((name.clone(), ty));
        }
    }
    pairs.sort_by(|(a, at), (b, bt)| a.cmp(b).then(at.cmp(bt)));
    Ok(pairs)
}
