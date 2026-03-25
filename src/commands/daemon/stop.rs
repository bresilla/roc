use clap::ArgMatches;

pub fn handle(_matches: ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    Err(crate::commands::daemon::unimplemented_message("stop").into())
}
