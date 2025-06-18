use clap::ArgMatches;
use crate::graph::RclGraphContext;
use anyhow::{Result, anyhow};

fn run_command(matches: ArgMatches) -> Result<()> {
    let topic_name = matches.get_one::<String>("topic_name")
        .ok_or_else(|| anyhow!("Topic name is required"))?;
    
    let verbose = matches.get_flag("verbose");
    
    // Create RCL context for graph queries
    let context = RclGraphContext::new()
        .map_err(|e| anyhow!("Failed to initialize RCL context: {}", e))?;
    
    // Get all topics and their types to find the type of the requested topic
    let topics_and_types = context.get_topic_names_and_types()
        .map_err(|e| anyhow!("Failed to get topic names and types: {}", e))?;
    
    // Find the topic type
    let topic_type = topics_and_types.iter()
        .find(|(name, _)| name == topic_name)
        .map(|(_, type_name)| type_name.clone())
        .ok_or_else(|| anyhow!("Topic '{}' not found", topic_name))?;
    
    // Get publisher and subscriber counts
    let publisher_count = context.count_publishers(topic_name)
        .map_err(|e| anyhow!("Failed to count publishers: {}", e))?;
    
    let subscriber_count = context.count_subscribers(topic_name)
        .map_err(|e| anyhow!("Failed to count subscribers: {}", e))?;
    
    // Print basic info
    println!("Type: {}", topic_type);
    println!("Publisher count: {}", publisher_count);
    println!("Subscription count: {}", subscriber_count);
    
    // Note: Verbose mode with detailed endpoint info is not yet implemented
    // due to complex RCL endpoint info struct initialization requirements
    if verbose {
        println!();
        println!("Note: Verbose mode with detailed endpoint information is not yet implemented.");
        println!("The --verbose flag is recognized but detailed endpoint info requires additional RCL API work.");
    }
    
    Ok(())
}

pub fn handle(matches: ArgMatches) {
    if let Err(e) = run_command(matches) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}