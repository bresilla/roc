use clap::ArgMatches;
use crate::graph::RclGraphContext;
use anyhow::Result;

fn run_command(matches: ArgMatches) -> Result<()> {
    // Create RCL graph context for direct API access
    let graph_context = RclGraphContext::new()
        .map_err(|e| anyhow::anyhow!("Failed to initialize RCL graph context: {}", e))?;

    // Get topic names using direct RCL API calls
    let topics = graph_context.get_topic_names()
        .map_err(|e| anyhow::anyhow!("Failed to get topic names: {}", e))?;

    // Handle --count-topics flag
    if matches.get_flag("count_topics") {
        println!("{}", topics.len());
        return Ok(());
    }

    // Handle --include-hidden-topics flag
    let filtered_topics: Vec<String> = if matches.get_flag("include_hidden_topics") {
        topics
    } else {
        // Filter out hidden topics (those starting with underscore)
        topics.into_iter()
            .filter(|topic| !topic.starts_with("/_"))
            .collect()
    };

    // Handle --show-types flag
    if matches.get_flag("show_types") {
        // Get topics with their type information
        let topics_with_types = graph_context.get_topics_with_types()
            .map_err(|e| anyhow::anyhow!("Failed to get topic types: {}", e))?;
        
        // Filter hidden topics if needed
        let filtered_topics: Vec<_> = if matches.get_flag("include_hidden_topics") {
            topics_with_types
        } else {
            topics_with_types.into_iter()
                .filter(|topic| !topic.name.starts_with("/_"))
                .collect()
        };
        
        // Display topics with types
        for topic in &filtered_topics {
            if topic.types.is_empty() {
                println!("{} [unknown type]", topic.name);
            } else if topic.types.len() == 1 {
                println!("{} [{}]", topic.name, topic.types[0]);
            } else {
                // Multiple types (rare but possible)
                println!("{} [{}]", topic.name, topic.types.join(", "));
            }
        }
        
        if filtered_topics.is_empty() {
            eprintln!("No topics found.");
        }
        
        return Ok(());
    }

    // Simple topic list (default behavior)
    for topic in &filtered_topics {
        println!("{}", topic);
    }

    // Handle other flags (for future implementation)
    if matches.get_flag("use_sim_time") {
        // TODO: Implement simulation time handling when needed
        eprintln!("Warning: --use-sim-time flag not yet implemented in direct RCL mode");
    }
    
    if matches.get_flag("no_daemon") {
        // TODO: Our implementation already bypasses daemon, so this is effectively handled
        // We could add logic here to ensure no daemon interaction if needed
    }
    
    if let Some(spin_time_value) = matches.get_one::<String>("spin_time") {
        // TODO: Implement spin time logic when needed for live topic discovery
        eprintln!("Warning: --spin-time {} flag not yet implemented in direct RCL mode", spin_time_value);
    }

    // Show helpful message if no topics found
    if filtered_topics.is_empty() {
        eprintln!("No topics found.");
    }

    Ok(())
}

pub fn handle(matches: ArgMatches) {
    match run_command(matches) {
        Ok(()) => {},
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}