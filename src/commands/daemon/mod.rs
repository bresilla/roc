use crate::commands::cli::handle_boxed_command_result;
use clap::ArgMatches;

pub(crate) fn unimplemented_message(subcommand: &str) -> String {
    format!(
        "roc daemon {subcommand} is not implemented yet. Use `ros2 daemon {subcommand}` directly for now."
    )
}

pub fn handle(matches: ArgMatches) {
    let result = match matches.subcommand() {
        Some(("start", args)) => start::handle(args.clone()),
        Some(("stop", args)) => stop::handle(args.clone()),
        Some(("status", args)) => status::handle(args.clone()),
        _ => Err("roc daemon is not implemented yet. Use `ros2 daemon` directly for now.".into()),
    };
    handle_boxed_command_result(result);
}

pub mod start;
pub mod status;
pub mod stop;

#[cfg(test)]
mod tests {
    use super::unimplemented_message;

    #[test]
    fn unimplemented_message_points_users_to_ros2_daemon() {
        assert_eq!(
            unimplemented_message("status"),
            "roc daemon status is not implemented yet. Use `ros2 daemon status` directly for now."
        );
    }
}
