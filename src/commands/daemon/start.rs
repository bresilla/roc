use crate::ui::{blocks, output};
use clap::ArgMatches;
use serde_json::json;

pub fn handle(matches: ArgMatches) {
    match output::OutputMode::from_matches(&matches) {
        output::OutputMode::Human => {
            blocks::print_section("Daemon Start");
            blocks::print_field("Result", "no action required");
            blocks::print_field("Mode", "direct DDS discovery");
            blocks::print_note("roc does not spawn a background daemon for native graph queries");
        }
        output::OutputMode::Plain => {
            println!("no-op");
        }
        output::OutputMode::Json => {
            output::print_json(&json!({
                "action": "start",
                "result": "no-op",
                "uses_daemon": false,
                "mode": "direct-dds",
            }))
            .ok();
        }
    }
}
