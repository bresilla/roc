use clap::ArgMatches;

pub fn handle(matches: ArgMatches) {
    match matches.subcommand() {
        Some(("create", args)) => {
            create::handle(args.clone());
        }
        Some(("list", args)) => {
            list::handle(args.clone());
        }
        Some(("info", args)) => {
            info::handle(args.clone());
        }
        Some(("build", args)) => {
            build::handle(args.clone());
        }
        _ => unreachable!("UNREACHABLE"),
    }
}

pub mod create;
pub mod list;
pub mod info;
pub mod build;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn get_subcommand_matches(args: Vec<String>, expected_subcommand: &str) -> clap::ArgMatches {
        let matches = crate::arguments::work::cmd()
            .try_get_matches_from(args)
            .unwrap();
        let (name, submatches) = matches.subcommand().unwrap();
        assert_eq!(name, expected_subcommand);
        submatches.clone()
    }

    #[test]
    fn create_then_list_and_info_work_in_temp_workspace() {
        let temp = tempdir().unwrap();
        let workspace = temp.path().to_path_buf();
        let src_dir = workspace.join("src");
        fs::create_dir_all(&src_dir).unwrap();

        let create_matches = get_subcommand_matches(
            vec![
                "work".to_string(),
                "create".to_string(),
                "demo_pkg".to_string(),
                "--destination_directory".to_string(),
                src_dir.display().to_string(),
                "--build_type".to_string(),
                "ament_cmake".to_string(),
                "--node_name".to_string(),
                "talker".to_string(),
            ],
            "create",
        );

        create::command::create_package_for_tests(create_matches).unwrap();
        assert!(workspace.join("src").join("demo_pkg").join("package.xml").exists());

        let list_matches = get_subcommand_matches(
            vec!["work".to_string(), "list".to_string()],
            "list",
        );

        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime
            .block_on(list::command::run_command_for_tests(
                list_matches,
                workspace.clone(),
            ))
            .unwrap();

        let info_matches = get_subcommand_matches(
            vec!["work".to_string(), "info".to_string(), "demo_pkg".to_string()],
            "info",
        );

        runtime
            .block_on(info::command::run_command_for_tests(info_matches, workspace))
            .unwrap();
    }
}
