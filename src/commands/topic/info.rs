use clap::ArgMatches;
use crate::graph::RclGraphContext;
use crate::arguments::topic::CommonTopicArgs;
use anyhow::{Result, anyhow};

fn run_command(matches: ArgMatches, common_args: CommonTopicArgs) -> Result<()> {
    let topic_name = matches.get_one::<String>("topic_name")
        .ok_or_else(|| anyhow!("Topic name is required"))?;
    
    let verbose = matches.get_flag("verbose");
    
    // Choose appropriate context creation
    // Note: Our implementation always does direct DDS discovery (daemon-free by design)
    // so --no-daemon doesn't change our behavior, but we acknowledge it for compatibility
    let create_context = || -> Result<RclGraphContext> {
        RclGraphContext::new()
            .map_err(|e| anyhow!("Failed to initialize RCL context: {}", e))
    };
    
    // Log a note about daemon usage if the flag is explicitly set
    if common_args.no_daemon {
        eprintln!("Note: roc always uses direct DDS discovery (equivalent to --no-daemon)");
    }
    
    // Get topic type
    let topic_type = {
        let context = create_context()?;
        
        let topics_and_types = context.get_topic_names_and_types()
            .map_err(|e| anyhow!("Failed to get topic names and types: {}", e))?;
        
        topics_and_types.iter()
            .find(|(name, _)| name == topic_name)
            .map(|(_, type_name)| type_name.clone())
            .ok_or_else(|| {
                let daemon_status = RclGraphContext::get_daemon_status();
                anyhow!("Topic '{}' not found. [{}]", topic_name, daemon_status)
            })?
    };
    
    // Get publisher count
    let publisher_count = {
        let context = create_context()?;
        context.count_publishers(topic_name)
            .map_err(|e| anyhow!("Failed to count publishers: {}", e))?
    };
    
    // Get subscriber count
    let subscriber_count = {
        let context = create_context()?;
        context.count_subscribers(topic_name)
            .map_err(|e| anyhow!("Failed to count subscribers: {}", e))?
    };
    
    println!("Type: {}", topic_type);
    println!("Publisher count: {}", publisher_count);
    println!("Subscription count: {}", subscriber_count);
    
    if verbose {
        // Get detailed publisher info
        let publishers_info = {
            let context = create_context()?;
            context.get_publishers_info(topic_name)
                .map_err(|e| anyhow!("Failed to get publishers info: {}", e))?
        };
        
        // Get detailed subscriber info
        let subscribers_info = {
            let context = create_context()?;
            context.get_subscribers_info(topic_name)
                .map_err(|e| anyhow!("Failed to get subscribers info: {}", e))?
        };

        println!("\nPublishers:");
        if publishers_info.is_empty() {
            println!("  <none>");
        } else {
            for pub_info in publishers_info {
                println!("  - Node name: {}", pub_info.node_name);
                println!("    Node namespace: {}", pub_info.node_namespace);
                println!("    Topic type: {}", pub_info.topic_type);
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
            }
        }
    }
    
    Ok(())
}

pub fn handle(matches: ArgMatches, common_args: CommonTopicArgs) {
    if let Err(e) = run_command(matches, common_args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}