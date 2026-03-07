use anyhow::Result;
use clap::{parser::ValueSource, Arg, ArgMatches};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Human,
    Plain,
    Json,
}

impl OutputMode {
    pub fn from_matches(matches: &ArgMatches) -> Self {
        match matches
            .get_one::<String>("output")
            .map(|value| value.as_str())
            .unwrap_or("human")
        {
            "plain" => Self::Plain,
            "json" => Self::Json,
            _ => Self::Human,
        }
    }

    pub fn from_matches_with_compat(matches: &ArgMatches, compatibility_plain: bool) -> Self {
        let explicit_output = matches
            .value_source("output")
            .map(|source| source != ValueSource::DefaultValue)
            .unwrap_or(false);

        if explicit_output {
            Self::from_matches(matches)
        } else if compatibility_plain {
            Self::Plain
        } else {
            Self::Human
        }
    }
}

pub fn arg() -> Arg {
    Arg::new("output")
        .long("output")
        .value_name("MODE")
        .value_parser(["human", "plain", "json"])
        .default_value("human")
        .help("Output mode: human, plain, or json")
}

pub fn print_json<T: Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

pub fn print_plain_section(title: &str) {
    println!("{title}:");
}

pub fn print_plain_field(label: &str, value: impl std::fmt::Display) {
    println!("{label}: {value}");
}

pub fn print_plain_status(label: &str, fields: &[(&str, String)]) {
    let mut rendered = String::new();
    for (index, (name, value)) in fields.iter().enumerate() {
        if index > 0 {
            rendered.push(' ');
        }
        rendered.push_str(&format!("{name}={value}"));
    }
    println!("{label}: {rendered}");
}

pub fn print_plain_multiline_field(label: &str, value: &str) {
    println!("{label}:");
    print!("{value}");
    if !value.ends_with('\n') {
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::{arg, OutputMode};
    use clap::Command;

    #[test]
    fn explicit_output_mode_overrides_compatibility_mode() {
        let matches = Command::new("test")
            .arg(arg())
            .try_get_matches_from(["test", "--output", "json"])
            .unwrap();

        assert_eq!(
            OutputMode::from_matches_with_compat(&matches, true),
            OutputMode::Json
        );
    }

    #[test]
    fn compatibility_mode_applies_when_output_is_not_explicitly_set() {
        let matches = Command::new("test").arg(arg()).get_matches_from(["test"]);

        assert_eq!(
            OutputMode::from_matches_with_compat(&matches, true),
            OutputMode::Plain
        );
    }
}
