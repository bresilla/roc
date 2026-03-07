use crate::ui::output;
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
                .arg(arg!(--recursive "Scan subdirectories recursively").action(ArgAction::SetTrue))
                .arg(output::arg()),
        )
        .subcommand(
            Command::new("info")
                .about("Show rosbag2 recording info")
                .aliases(["i", "show"])
                .arg(arg!(<PATH> "Bag directory containing metadata.yaml").required(true))
                .arg(output::arg()),
        )
        .subcommand(
            Command::new("record")
                .about("Record messages into an MCAP file")
                .aliases(["r", "rec"])
                .arg(arg!([topics] ... "Topics to record"))
                .arg(arg!(-o --output <OUTPUT> "Output MCAP file path"))
                .arg(arg!(--all "Record all topics").action(ArgAction::SetTrue))
                .arg(arg!(--type <TYPE> "Message type (single-topic override)").required(false))
                .arg(
                    arg!(--separated "Write one MCAP per topic when recording multiple topics")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("play")
                .about("Play back MCAP files")
                .aliases(["p"])
                .arg(arg!(<PATHS> ... "MCAP files to play").required(true))
                .arg(arg!(--rate <RATE> "Playback rate multiplier (default: 1.0)"))
                .arg(arg!(--loop "Loop playback").action(ArgAction::SetTrue)),
        )
}
