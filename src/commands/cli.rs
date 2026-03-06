use anyhow::Result as AnyhowResult;
use clap::ArgMatches;
use colored::Colorize;
use std::error::Error;
use std::future::Future;

pub type CommandResult<E = Box<dyn std::error::Error>> = Result<(), E>;

pub fn required_string<'a>(matches: &'a ArgMatches, name: &str) -> Result<&'a str, Box<dyn Error>> {
    matches
        .get_one::<String>(name)
        .map(|value| value.as_str())
        .ok_or_else(|| format!("Required argument '{name}' is missing").into())
}

pub fn joined_values(matches: &ArgMatches, name: &str) -> Option<String> {
    matches
        .get_many::<String>(name)
        .map(|values| {
            values
                .map(|value| value.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .map(|joined| joined.trim().to_string())
        .filter(|joined| !joined.is_empty())
}

pub fn print_error_and_exit(error: impl std::fmt::Display) -> ! {
    eprintln!("{} {}", "Error:".bright_red().bold(), error);
    std::process::exit(1);
}

fn is_broken_pipe(error: &(dyn Error + 'static)) -> bool {
    let mut current = Some(error);
    while let Some(source) = current {
        if let Some(io_error) = source.downcast_ref::<std::io::Error>() {
            if io_error.kind() == std::io::ErrorKind::BrokenPipe {
                return true;
            }
        }
        current = source.source();
    }
    false
}

pub fn handle_anyhow_result(result: AnyhowResult<()>) {
    if let Err(error) = result {
        if let Some(io_error) = error.downcast_ref::<std::io::Error>() {
            if io_error.kind() == std::io::ErrorKind::BrokenPipe {
                return;
            }
        }
        print_error_and_exit(error);
    }
}

pub fn handle_boxed_command_result(result: CommandResult) {
    if let Err(error) = result {
        if is_broken_pipe(error.as_ref()) {
            return;
        }
        print_error_and_exit(error);
    }
}

pub fn run_async_command<F, E>(future: F)
where
    F: Future<Output = CommandResult<E>>,
    E: std::fmt::Display,
{
    let runtime = tokio::runtime::Runtime::new().unwrap_or_else(|error| {
        print_error_and_exit(format!("Failed to create async runtime: {error}"))
    });

    if let Err(error) = runtime.block_on(future) {
        print_error_and_exit(error);
    }
}
