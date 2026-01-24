use anyhow::{anyhow, Result};

use crate::shared::graph_context::RclGraphContext;
use crate::shared::graph_types::{TopicEndpointInfo, TopicInfo};

/// Get all topics in the ROS graph.
pub fn get_topic_names(context: &RclGraphContext) -> Result<Vec<String>> {
    if !context.is_valid() {
        return Err(anyhow!("RCL context is not valid"));
    }

    let map = context.node().get_topic_names_and_types()?;
    let mut names: Vec<String> = map.keys().cloned().collect();
    names.sort();
    Ok(names)
}

/// Get all topics in the ROS graph with their type information.
pub fn get_topics_with_types(context: &RclGraphContext) -> Result<Vec<TopicInfo>> {
    if !context.is_valid() {
        return Err(anyhow!("RCL context is not valid"));
    }

    let map = context.node().get_topic_names_and_types()?;
    let mut topics: Vec<TopicInfo> = map
        .into_iter()
        .map(|(name, types)| TopicInfo { name, types })
        .collect();
    topics.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(topics)
}

/// Get all topics and their types as tuples.
pub fn get_topic_names_and_types(context: &RclGraphContext) -> Result<Vec<(String, String)>> {
    if !context.is_valid() {
        return Err(anyhow!("RCL context is not valid"));
    }

    let map = context.node().get_topic_names_and_types()?;
    let mut pairs = Vec::new();
    for (name, types) in map {
        for ty in types {
            pairs.push((name.clone(), ty));
        }
    }
    pairs.sort_by(|(a, at), (b, bt)| a.cmp(b).then(at.cmp(bt)));
    Ok(pairs)
}

/// Count the number of publishers for a given topic.
pub fn count_publishers(context: &RclGraphContext, topic_name: &str) -> Result<usize> {
    if !context.is_valid() {
        return Err(anyhow!("RCL context is not valid"));
    }
    Ok(context.node().count_publishers(topic_name)? as usize)
}

/// Count the number of subscriptions for a given topic.
pub fn count_subscribers(context: &RclGraphContext, topic_name: &str) -> Result<usize> {
    if !context.is_valid() {
        return Err(anyhow!("RCL context is not valid"));
    }
    Ok(context.node().count_subscriptions(topic_name)? as usize)
}

/// Get endpoint information for publishers to a topic.
pub fn get_publishers_info(
    context: &RclGraphContext,
    topic_name: &str,
) -> Result<Vec<TopicEndpointInfo>> {
    if !context.is_valid() {
        return Err(anyhow!("RCL context is not valid"));
    }

    let infos = context.node().get_publishers_info_by_topic(topic_name)?;
    Ok(infos
        .into_iter()
        .map(|i| TopicEndpointInfo {
            node_name: i.node_name,
            node_namespace: i.node_namespace,
            topic_type: i.topic_type,
        })
        .collect())
}

/// Get endpoint information for subscriptions to a topic.
pub fn get_subscribers_info(
    context: &RclGraphContext,
    topic_name: &str,
) -> Result<Vec<TopicEndpointInfo>> {
    if !context.is_valid() {
        return Err(anyhow!("RCL context is not valid"));
    }

    let infos = context.node().get_subscriptions_info_by_topic(topic_name)?;
    Ok(infos
        .into_iter()
        .map(|i| TopicEndpointInfo {
            node_name: i.node_name,
            node_namespace: i.node_namespace,
            topic_type: i.topic_type,
        })
        .collect())
}
