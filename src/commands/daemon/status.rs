use crate::ui::{blocks, output};
use clap::ArgMatches;
use serde_json::json;

pub fn handle(matches: ArgMatches) {
    match output::OutputMode::from_matches(&matches) {
        output::OutputMode::Human => {
            blocks::print_section("Daemon");
            blocks::print_field("Mode", "direct DDS discovery");
            blocks::print_field("Uses Daemon", "false");
            blocks::print_field("Status", "not required");
            blocks::print_note("roc queries the ROS graph directly and does not rely on a background daemon");
        }
        output::OutputMode::Plain => {
            println!("direct-dds");
        }
        output::OutputMode::Json => {
            output::print_json(&json!({
                "mode": "direct-dds",
                "uses_daemon": false,
                "status": "not-required",
            }))
            .ok();
        }
    }
}
