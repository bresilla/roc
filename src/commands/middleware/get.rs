use crate::commands::middleware::{current_implementation, discover_implementations};
use crate::ui::{blocks, output};
use clap::ArgMatches;
use serde_json::json;

pub fn handle(matches: ArgMatches) {
    let output_mode = output::OutputMode::from_matches(&matches);
    let current = current_implementation();
    let available = discover_implementations();
    let source = if current.is_some() {
        "env"
    } else {
        "runtime-default"
    };

    match output_mode {
        output::OutputMode::Human => {
            blocks::print_section("Middleware");
            match current.as_deref() {
                Some(current) => {
                    blocks::print_field("Current", current);
                    blocks::print_field("Source", "RMW_IMPLEMENTATION");
                }
                None => {
                    blocks::print_field("Current", "<runtime default>");
                    blocks::print_field("Source", "ROS 2 runtime selection");
                }
            }
            if !available.is_empty() {
                blocks::print_field("Discovered", available.len());
            }
        }
        output::OutputMode::Plain => {
            println!("{}", current.unwrap_or_else(|| "<runtime default>".to_string()));
        }
        output::OutputMode::Json => {
            output::print_json(&json!({
                "current": current,
                "source": source,
                "implementations": available,
            }))
            .ok();
        }
    }
}
