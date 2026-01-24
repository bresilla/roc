use clap::{arg, ArgAction, Command};

pub fn cmd() -> Command {
    Command::new("bag")
        .about("Various rosbag subcommands")
        .aliases(&["b"])
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("list")
                .about("List rosbag2 recordings")
                .aliases(["l", "ls"])
                .arg(arg!([PATH] "Directory to scan (default: .)").required(false))
                .arg(
                    arg!(--recursive "Scan subdirectories recursively").action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("info")
                .about("Show rosbag2 recording info")
                .aliases(["i", "show"])
                .arg(arg!(<PATH> "Bag directory containing metadata.yaml").required(true)),
        )
        .subcommand(
            Command::new("record")
                .about("Record messages into a bag (ROS 2 CLI wrapper for now)")
                .aliases(["r", "rec"])
                .arg(arg!([topics] ... "Topics to record"))
                .arg(arg!(-o --output <OUTPUT> "Output directory name"))
                .arg(arg!(--all "Record all topics").action(ArgAction::SetTrue)),
        )
        .subcommand(
            Command::new("play")
                .about("Play back a bag (WIP)")
                .aliases(["p"])
                .arg(arg!(<PATH> "Bag directory to play").required(true)),
        )
}
