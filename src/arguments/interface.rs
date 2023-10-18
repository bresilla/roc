use clap::{arg, Command, Arg, ArgAction};

pub fn cmd() -> Command {
    Command::new("interface")
        .about("Various interface subcommands")
        .aliases(&["i", "int"])
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("list")
            .about("List all interface types available")
            .aliases(["l", "ls"])
            .arg(
                Arg::new("messages")
                .long("messages")
                .short('m')
                .aliases(&["msgs"])
                .visible_aliases(&["all"])
                .help("Print out only the message types")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(&["services", "actions"])
            )
            .arg(
                Arg::new("services")
                .long("services")
                .short('s')
                .aliases(&["srvs"])
                .help("Print out only the service types")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(&["messages", "actions"])
            )
            .arg(
                Arg::new("actions")
                .long("actions")
                .short('a')
                .aliases(&["acts"])
                .help("Print out only the action types")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(&["messages", "services"])
            )
        )
        .subcommand(
            Command::new("package")
            .about("Output a list of available interface types within one package")
            .aliases(["p", "pkg"])
            .arg_required_else_help(true)
            .arg(
                Arg::new("package_name")
                .help("Name of the ROS package (e.g. 'example_interfaces')")
                .required(true)
                .value_name("PACKAGE_NAME")
            )
        )
        .subcommand(
            Command::new("all")
            .about("Output a list of packages that provide interfaces")
            .aliases(["a", "packages"])
            .arg(
                Arg::new("messages")
                .long("messages")
                .short('m')
                .aliases(&["msgs"])
                .visible_aliases(&["all"])
                .help("Print out only the message types")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(&["services", "actions"])
            )
            .arg(
                Arg::new("services")
                .long("services")
                .short('s')
                .aliases(&["srvs"])
                .help("Print out only the service types")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(&["messages", "actions"])
            )
            .arg(
                Arg::new("actions")
                .long("actions")
                .short('a')
                .aliases(&["acts"])
                .help("Print out only the action types")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(&["messages", "services"])
            )
        )
        .subcommand(
            Command::new("show")
            .about("Show the interface definition for a given type")
            .aliases(["s", "info"])
            .arg_required_else_help(true)
            .arg(
                arg!(<TYPE> "Show an interface definition (e.g. 'example_interfaces/msg/String'). Passing '-' reads the argument from stdin (e.g. 'ros2 topic type /chatter | ros2 interface show -').")
                .required(true)
            )
            .arg(arg!(--all_comments "Show all comments, including for nested interface definitions"))
            .arg(arg!(--no_comments "Show no comments or whitespace"))
        )
        .subcommand(
            Command::new("model")
            .about("Output an interface model/prototype")
            .aliases(["m", "prototype", "proto"])
            .arg_required_else_help(true)
            .arg(
                Arg::new("type")
                .help("Show an interface definition (e.g. 'example_interfaces/msg/String')")
                .required(true)
                .value_name("TYPE")
            )
            .arg(
                Arg::new("no_quotes")
                .long("no-quotes")
                .help("Do not output outer quotes")
                .action(ArgAction::SetTrue)
            )             
        )
}