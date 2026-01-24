use clap::{arg, Command};

pub fn cmd() -> Command {
    Command::new("frame")
        .about("Various transforms subcommands [WIP]")
        .aliases(&["f", "tf"])
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("echo")
                .about("Print transforms from the tree to screen")
                .aliases(["e", "cat"])
                .arg_required_else_help(true)
                .arg(arg!(<FRAME_ID> "Name of the frame id (e.g. 'odom')").required(true))
                .arg(
                    arg!(<CHILD_FRAME_ID> "Name of the child frame id (e.g. 'base_link')")
                        .required(true),
                )
                .arg(arg!(--rate <RATE> "Rate at which to display transforms. Default: 10.0"))
                .arg(arg!(--once "Print first message and exit")),
        )
        .subcommand(
            Command::new("list")
                .about("Output a list of available frames")
                .aliases(["l", "ls"])
                .arg(arg!(-a --all "Display all frames even hidden ones"))
                .arg(arg!(-c --count_frames "Only display the number of frames discovered")),
        )
        .subcommand(
            Command::new("info")
                .about("Print information about a frame")
                .aliases(["i", "show"])
                .arg_required_else_help(true)
                .arg(
                    arg!(<FRAME_NAME> "Name of the frame to get info (e.g. 'base_link')")
                        .required(true),
                )
                .arg(arg!(--include_hidden_services "Include hidden services"))
                .arg(
                    arg!(--export_dot <EXPORT_DOT> "Export the frame tree to a dot file")
                        .conflicts_with("export_json")
                        .conflicts_with("export_yaml"),
                )
                .arg(
                    arg!(--export_json <EXPORT_JSON> "Export the frame tree to a json file")
                        .conflicts_with("export_dot")
                        .conflicts_with("export_yaml"),
                )
                .arg(
                    arg!(--export_yaml <EXPORT_YAML> "Export the frame tree to a yaml file")
                        .conflicts_with("export_dot")
                        .conflicts_with("export_json"),
                )
                .arg(arg!(--export_image <EXPORT_IMAGE> "Export the frame tree to an image file")),
        )
        .subcommand(
            Command::new("pub")
                .about("Publish a static transform")
                .aliases(["p", "publish"])
                .arg_required_else_help(true)
                .arg(arg!(<FRAME_ID> "Name of the frame id (e.g. 'odom')").required(true))
                .arg(
                    arg!(<CHILD_FRAME_ID> "Name of the child frame id (e.g. 'base_link')")
                        .required(true),
                )
                .arg(arg!(--x <X> "x component of translation"))
                .arg(arg!(--y <Y> "y component of translation"))
                .arg(arg!(--z <Z> "z component of translation"))
                .arg(
                    arg!(--qx <QX> "x component of quaternion rotation")
                        .conflicts_with("roll")
                        .conflicts_with("pitch")
                        .conflicts_with("yaw"),
                )
                .arg(
                    arg!(--qy <QY> "y component of quaternion rotation")
                        .conflicts_with("roll")
                        .conflicts_with("pitch")
                        .conflicts_with("yaw"),
                )
                .arg(
                    arg!(--qz <QZ> "z component of quaternion rotation")
                        .conflicts_with("roll")
                        .conflicts_with("pitch")
                        .conflicts_with("yaw"),
                )
                .arg(
                    arg!(--qw <QW> "w component of quaternion rotation")
                        .conflicts_with("roll")
                        .conflicts_with("pitch")
                        .conflicts_with("yaw"),
                )
                .arg(
                    arg!(--roll <ROLL> "roll component Euler rotation")
                        .conflicts_with("qx")
                        .conflicts_with("qy")
                        .conflicts_with("qz")
                        .conflicts_with("qw"),
                )
                .arg(
                    arg!(--pitch <PITCH> "pitch component Euler rotation")
                        .conflicts_with("qx")
                        .conflicts_with("qy")
                        .conflicts_with("qz")
                        .conflicts_with("qw"),
                )
                .arg(
                    arg!(--yaw <YAW> "yaw component Euler rotation")
                        .conflicts_with("qx")
                        .conflicts_with("qy")
                        .conflicts_with("qz")
                        .conflicts_with("qw"),
                ),
        )
}
