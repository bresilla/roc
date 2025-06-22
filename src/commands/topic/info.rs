use clap::ArgMatches;
use crate::graph::RclGraphContext;
use crate::arguments::topic::CommonTopicArgs;
use anyhow::{Result, anyhow};
use colored::*;

fn run_command(matches: ArgMatches, common_args: CommonTopicArgs) -> Result<()> {
    let topic_name = matches.get_one::<String>("topic_name")
        .ok_or_else(|| anyhow!("Topic name is required"))?;
    
    let verbose = matches.get_flag("verbose");
    
    // Create a single RCL context for all operations
    // Note: Our implementation always does direct DDS discovery (daemon-free by design)
    // so --no-daemon doesn't change our behavior, but we acknowledge it for compatibility
    let context = RclGraphContext::new()
        .map_err(|e| anyhow!("Failed to initialize RCL context: {}", e))?;
    
    // Log a note about daemon usage if the flag is explicitly set
    if common_args.no_daemon {
        eprintln!("Note: roc always uses direct DDS discovery (equivalent to --no-daemon)");
    }
    
    // Get topic type
    let topic_type = {
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
    let publisher_count = context.count_publishers(topic_name)
        .map_err(|e| anyhow!("Failed to count publishers: {}", e))?;
    
    // Get subscriber count
    let subscriber_count = context.count_subscribers(topic_name)
        .map_err(|e| anyhow!("Failed to count subscribers: {}", e))?;
    
    if common_args.ros_style {
        // Original ROS2 CLI style
        println!("Type: {}", topic_type);
        println!("Publisher count: {}", publisher_count);
        println!("Subscription count: {}", subscriber_count);
    } else {
        // Enhanced colored output
        println!("{} {}", "Type:".bright_yellow().bold(), topic_type.bright_cyan());
        println!("{} {}", "Publisher count:".bright_yellow().bold(), 
                 if publisher_count > 0 { publisher_count.to_string().bright_green() } else { publisher_count.to_string().red() });
        println!("{} {}", "Subscription count:".bright_yellow().bold(), 
                 if subscriber_count > 0 { subscriber_count.to_string().bright_green() } else { subscriber_count.to_string().red() });
    }
    
    if verbose {
        // Get detailed publisher info
        let publishers_info = context.get_publishers_info(topic_name)
            .map_err(|e| anyhow!("Failed to get publishers info: {}", e))?;
        
        // Allow some time for any internal RCL state to settle after the first call
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        // Get detailed subscriber info
        let subscribers_info = context.get_subscribers_info(topic_name)
            .map_err(|e| anyhow!("Failed to get subscribers info: {}", e))?;

        if common_args.ros_style {
            // Original ROS2 CLI style
            println!("\nPublishers:");
            if publishers_info.is_empty() {
                println!("  <none>");
            } else {
                for pub_info in publishers_info {
                    let endpoint_type = match pub_info.endpoint_type {
                        crate::graph::EndpointType::Publisher => "PUBLISHER",
                        crate::graph::EndpointType::Subscription => "SUBSCRIPTION",
                        crate::graph::EndpointType::Invalid => "INVALID",
                    };
                    
                    println!("  - Node name: {}", pub_info.node_name);
                    println!("    Node namespace: {}", pub_info.node_namespace);
                    println!("    Topic type: {}", pub_info.topic_type);
                    println!("    Topic type hash: RIHS01_{}", pub_info.topic_type_hash);
                    println!("    Endpoint type: {}", endpoint_type);
                    println!("    GID: {}", format_gid(&pub_info.gid));
                    println!("    QoS profile:");
                    println!("      Reliability: {}", pub_info.qos_profile.reliability.to_string());
                    println!("      History ({}): {}", pub_info.qos_profile.history.to_string(), pub_info.qos_profile.depth);
                    println!("      Durability: {}", pub_info.qos_profile.durability.to_string());
                    println!("      Lifespan: {}", pub_info.qos_profile.format_duration(pub_info.qos_profile.lifespan_sec, pub_info.qos_profile.lifespan_nsec));
                    println!("      Deadline: {}", pub_info.qos_profile.format_duration(pub_info.qos_profile.deadline_sec, pub_info.qos_profile.deadline_nsec));
                    println!("      Liveliness: {}", pub_info.qos_profile.liveliness.to_string());
                    println!("      Liveliness lease duration: {}", pub_info.qos_profile.format_duration(pub_info.qos_profile.liveliness_lease_duration_sec, pub_info.qos_profile.liveliness_lease_duration_nsec));
                }
            }
        } else {
            // Enhanced colored output
            println!();
            println!("{}", "Publishers:".bright_magenta().bold());
            if publishers_info.is_empty() {
                println!("  {}", "<none>".bright_black());
            } else {
                for pub_info in publishers_info {
                    let endpoint_type = match pub_info.endpoint_type {
                        crate::graph::EndpointType::Publisher => "PUBLISHER".bright_green(),
                        crate::graph::EndpointType::Subscription => "SUBSCRIPTION".bright_blue(),
                        crate::graph::EndpointType::Invalid => "INVALID".red(),
                    };
                    
                    println!("  • {}: {}", "Node name".bright_yellow(), pub_info.node_name.bright_white());
                    println!("    {}: {}", "Node namespace".bright_yellow(), pub_info.node_namespace.bright_white());
                    println!("    {}: {}", "Topic type".bright_yellow(), pub_info.topic_type.bright_cyan());
                    println!("    {}: RIHS01_{}", "Topic type hash".bright_yellow(), pub_info.topic_type_hash.bright_black());
                    println!("    {}: {}", "Endpoint type".bright_yellow(), endpoint_type);
                    println!("    {}: {}", "GID".bright_yellow(), format_gid(&pub_info.gid).bright_black());
                    println!("    {}:", "QoS profile".bright_yellow());
                    println!("      {}: {}", "Reliability".cyan(), pub_info.qos_profile.reliability.to_string().bright_white());
                    println!("      {} ({}): {}", "History".cyan(), pub_info.qos_profile.history.to_string().bright_white(), pub_info.qos_profile.depth.to_string().bright_white());
                    println!("      {}: {}", "Durability".cyan(), pub_info.qos_profile.durability.to_string().bright_white());
                    println!("      {}: {}", "Lifespan".cyan(), pub_info.qos_profile.format_duration(pub_info.qos_profile.lifespan_sec, pub_info.qos_profile.lifespan_nsec).bright_white());
                    println!("      {}: {}", "Deadline".cyan(), pub_info.qos_profile.format_duration(pub_info.qos_profile.deadline_sec, pub_info.qos_profile.deadline_nsec).bright_white());
                    println!("      {}: {}", "Liveliness".cyan(), pub_info.qos_profile.liveliness.to_string().bright_white());
                    println!("      {}: {}", "Liveliness lease duration".cyan(), pub_info.qos_profile.format_duration(pub_info.qos_profile.liveliness_lease_duration_sec, pub_info.qos_profile.liveliness_lease_duration_nsec).bright_white());
                    println!();
                }
            }
        }

        if common_args.ros_style {
            // Original ROS2 CLI style
            println!("\nSubscribers:");
            if subscribers_info.is_empty() {
                println!("  <none>");
            } else {
                for sub_info in subscribers_info {
                    let endpoint_type = match sub_info.endpoint_type {
                        crate::graph::EndpointType::Publisher => "PUBLISHER",
                        crate::graph::EndpointType::Subscription => "SUBSCRIPTION", 
                        crate::graph::EndpointType::Invalid => "INVALID",
                    };
                    
                    println!("  - Node name: {}", sub_info.node_name);
                    println!("    Node namespace: {}", sub_info.node_namespace);
                    println!("    Topic type: {}", sub_info.topic_type);
                    println!("    Topic type hash: RIHS01_{}", sub_info.topic_type_hash);
                    println!("    Endpoint type: {}", endpoint_type);
                    println!("    GID: {}", format_gid(&sub_info.gid));
                    println!("    QoS profile:");
                    println!("      Reliability: {}", sub_info.qos_profile.reliability.to_string());
                    println!("      History ({}): {}", sub_info.qos_profile.history.to_string(), sub_info.qos_profile.depth);
                    println!("      Durability: {}", sub_info.qos_profile.durability.to_string());
                    println!("      Lifespan: {}", sub_info.qos_profile.format_duration(sub_info.qos_profile.lifespan_sec, sub_info.qos_profile.lifespan_nsec));
                    println!("      Deadline: {}", sub_info.qos_profile.format_duration(sub_info.qos_profile.deadline_sec, sub_info.qos_profile.deadline_nsec));
                    println!("      Liveliness: {}", sub_info.qos_profile.liveliness.to_string());
                    println!("      Liveliness lease duration: {}", sub_info.qos_profile.format_duration(sub_info.qos_profile.liveliness_lease_duration_sec, sub_info.qos_profile.liveliness_lease_duration_nsec));
                }
            }
        } else {
            // Enhanced colored output
            println!("{}", "Subscribers:".bright_magenta().bold());
            if subscribers_info.is_empty() {
                println!("  {}", "<none>".bright_black());
            } else {
                for sub_info in subscribers_info {
                    let endpoint_type = match sub_info.endpoint_type {
                        crate::graph::EndpointType::Publisher => "PUBLISHER".bright_green(),
                        crate::graph::EndpointType::Subscription => "SUBSCRIPTION".bright_blue(), 
                        crate::graph::EndpointType::Invalid => "INVALID".red(),
                    };
                    
                    println!("  • {}: {}", "Node name".bright_yellow(), sub_info.node_name.bright_white());
                    println!("    {}: {}", "Node namespace".bright_yellow(), sub_info.node_namespace.bright_white());
                    println!("    {}: {}", "Topic type".bright_yellow(), sub_info.topic_type.bright_cyan());
                    println!("    {}: RIHS01_{}", "Topic type hash".bright_yellow(), sub_info.topic_type_hash.bright_black());
                    println!("    {}: {}", "Endpoint type".bright_yellow(), endpoint_type);
                    println!("    {}: {}", "GID".bright_yellow(), format_gid(&sub_info.gid).bright_black());
                    println!("    {}:", "QoS profile".bright_yellow());
                    println!("      {}: {}", "Reliability".cyan(), sub_info.qos_profile.reliability.to_string().bright_white());
                    println!("      {} ({}): {}", "History".cyan(), sub_info.qos_profile.history.to_string().bright_white(), sub_info.qos_profile.depth.to_string().bright_white());
                    println!("      {}: {}", "Durability".cyan(), sub_info.qos_profile.durability.to_string().bright_white());
                    println!("      {}: {}", "Lifespan".cyan(), sub_info.qos_profile.format_duration(sub_info.qos_profile.lifespan_sec, sub_info.qos_profile.lifespan_nsec).bright_white());
                    println!("      {}: {}", "Deadline".cyan(), sub_info.qos_profile.format_duration(sub_info.qos_profile.deadline_sec, sub_info.qos_profile.deadline_nsec).bright_white());
                    println!("      {}: {}", "Liveliness".cyan(), sub_info.qos_profile.liveliness.to_string().bright_white());
                    println!("      {}: {}", "Liveliness lease duration".cyan(), sub_info.qos_profile.format_duration(sub_info.qos_profile.liveliness_lease_duration_sec, sub_info.qos_profile.liveliness_lease_duration_nsec).bright_white());
                    println!();
                }
            }
        }
    }
    
    Ok(())
}

// Helper function to format GID
fn format_gid(gid: &[u8]) -> String {
    gid.iter().map(|b| format!("{:02x}", b)).collect::<Vec<String>>().join(".")
}

pub fn handle(matches: ArgMatches, common_args: CommonTopicArgs) {
    if let Err(e) = run_command(matches, common_args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}