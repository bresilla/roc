use clap::Command;

pub fn cmd() -> Command {
    Command::new("daemon")
        .about("Experimental daemon placeholders; these subcommands are not implemented yet")
        .aliases(&["d"])
        .subcommand_required(true)
        .arg_required_else_help(true)
}

#[cfg(test)]
mod tests {
    use super::cmd;

    #[test]
    fn daemon_help_marks_command_as_unimplemented() {
        let mut command = cmd();
        let mut buffer = Vec::new();
        command.write_long_help(&mut buffer).unwrap();
        let help = String::from_utf8(buffer).unwrap();

        assert!(help.contains("not implemented yet"));
    }
}
