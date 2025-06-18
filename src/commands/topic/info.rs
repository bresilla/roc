use clap::ArgMatches;
use crate::graph::RclGraphContext;
use anyhow::{Result, anyhow};

fn run_command(matches: ArgMatches) -> Result<()> {
    let topic_name = matches.get_one::<String>("topic_name")
        .ok_or_else(|| anyhow!("Topic name is required"))?;
    
    let verbose = matches.get_flag("verbose");
    
    let context = RclGraphContext::new()
        .map_err(|e| anyhow!("Failed to initialize RCL context: {}", e))?;
    
    let topics_and_types = context.get_topic_names_and_types()
        .map_err(|e| anyhow!("Failed to get topic names and types: {}", e))?;
    
    let topic_type = topics_and_types.iter()
        .find(|(name, _)| name == topic_name)
        .map(|(_, type_name)| type_name.clone())
        .ok_or_else(|| anyhow!("Topic '{}' not found", topic_name))?;
    
    let publisher_count = context.count_publishers(topic_name)
        .map_err(|e| anyhow!("Failed to count publishers: {}", e))?;
    
    let subscriber_count = context.count_subscribers(topic_name)
        .map_err(|e| anyhow!("Failed to count subscribers: {}", e))?;
    
    println!("Type: {}", topic_type);
    println!("Publisher count: {}", publisher_count);
    println!("Subscription count: {}", subscriber_count);
    
    if verbose {
        let publishers_info = context.get_publishers_info(topic_name)
            .map_err(|e| anyhow!("Failed to get publishers info: {}", e))?;
        
        let subscribers_info = context.get_subscribers_info(topic_name)
            .map_err(|e| anyhow!("Failed to get subscribers info: {}", e))?;

        println!("\nPublishers:");
        if publishers_info.is_empty() {
            println!("  <none>");
        } else {
            for pub_info in publishers_info {
                println!("  - Node name: {}", pub_info.node_name);
                println!("    Node namespace: {}", pub_info.node_namespace);
                println!("    Topic type: {}", pub_info.topic_type);
                // GID and QoS are omitted for now
            }
        }

        println!("\nSubscribers:");
        if subscribers_info.is_empty() {
            println!("  <none>");
        } else {
            for sub_info in subscribers_info {
                println!("  - Node name: {}", sub_info.node_name);
                println!("    Node namespace: {}", sub_info.node_namespace);
                println!("    Topic type: {}", sub_info.topic_type);
                // GID and QoS are omitted for now
            }
        }
    }
    
    Ok(())
}

pub fn handle(matches: ArgMatches) {
    if let Err(e) = run_command(matches) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}