use crate::commands::middleware::{current_implementation, discover_implementations};
use crate::ui::{blocks, output, table};
use clap::ArgMatches;
use serde_json::json;

pub fn handle(matches: ArgMatches) {
    let output_mode = output::OutputMode::from_matches(&matches);
    let implementations = discover_implementations();
    let current = current_implementation();

    match output_mode {
        output::OutputMode::Human => {
            blocks::print_section("Middleware");
            if let Some(current) = current.as_deref() {
                blocks::print_field("Current", current);
            } else {
                blocks::print_field("Current", "<runtime default>");
            }

            if implementations.is_empty() {
                blocks::eprint_warning("No RMW implementations were discovered in the current ROS prefixes");
                return;
            }

            println!();
            blocks::print_section("Available");
            let rows = implementations
                .iter()
                .map(|implementation| {
                    vec![
                        implementation.clone(),
                        if current.as_deref() == Some(implementation.as_str()) {
                            "yes".to_string()
                        } else {
                            String::new()
                        },
                    ]
                })
                .collect();
            table::print_table(&["Implementation", "Selected"], rows);
            blocks::print_total(implementations.len(), "implementation", "implementations");
        }
        output::OutputMode::Plain => {
            for implementation in &implementations {
                println!("{implementation}");
            }
        }
        output::OutputMode::Json => {
            output::print_json(&json!({
                "current": current,
                "implementations": implementations,
            }))
            .ok();
        }
    }
}
