use crate::ui::{blocks, output};
use clap::ArgMatches;
use serde_json::json;

pub fn handle(matches: ArgMatches) {
    match output::OutputMode::from_matches(&matches) {
        output::OutputMode::Human => {
            blocks::print_section("Daemon Stop");
            blocks::print_field("Result", "no daemon running");
            blocks::print_field("Mode", "direct DDS discovery");
            blocks::print_note("roc has no native daemon process to stop");
        }
        output::OutputMode::Plain => {
            println!("no-op");
        }
        output::OutputMode::Json => {
            output::print_json(&json!({
                "action": "stop",
                "result": "no-op",
                "uses_daemon": false,
                "mode": "direct-dds",
            }))
            .ok();
        }
    }
}
