use crate::commands::cli::handle_boxed_command_result;
use clap::ArgMatches;
use colored::Colorize;
use roxmltree::Document;
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
struct ResultConfig {
    result_base: PathBuf,
    all: bool,
    verbose: bool,
    result_files_only: bool,
    delete: bool,
    delete_yes: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResultStatus {
    Passed,
    Failed,
    Error,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct JUnitCounts {
    tests: u64,
    failures: u64,
    errors: u64,
    skipped: u64,
}

#[derive(Debug, Clone)]
struct ResultEntry {
    package_name: String,
    path: PathBuf,
    status: ResultStatus,
    counts: Option<JUnitCounts>,
    detail: Option<String>,
    failures: Vec<FailureDetail>,
}

impl ResultEntry {
    fn matches_default_filter(&self) -> bool {
        self.status != ResultStatus::Passed
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FailureDetail {
    name: String,
    message: String,
}

fn config_from_matches(matches: &ArgMatches) -> Result<ResultConfig, Box<dyn std::error::Error>> {
    let workspace_root = std::env::current_dir()?;
    let mut result_base = matches
        .get_one::<String>("test_result_base")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root.join("build"));
    if !result_base.is_absolute() {
        result_base = workspace_root.join(result_base);
    }

    Ok(ResultConfig {
        result_base,
        all: matches.get_flag("all"),
        verbose: matches.get_flag("verbose"),
        result_files_only: matches.get_flag("result_files_only"),
        delete: matches.get_flag("delete"),
        delete_yes: matches.get_flag("delete_yes"),
    })
}

fn parse_result_xml(
    xml_path: &Path,
) -> Result<(JUnitCounts, Vec<FailureDetail>), Box<dyn std::error::Error>> {
    let xml = fs::read_to_string(xml_path)?;
    let doc = Document::parse(&xml)?;
    let root = doc.root_element();
    let mut counts = JUnitCounts::default();
    let mut failures = Vec::new();

    match root.tag_name().name() {
        "Site" => {
            for testing in root
                .descendants()
                .filter(|node| node.has_tag_name("Testing"))
            {
                for test in testing.children().filter(|node| node.has_tag_name("Test")) {
                    counts.tests += 1;
                    match test.attribute("Status").unwrap_or("failed") {
                        "passed" => {}
                        "notrun" => counts.skipped += 1,
                        _ => {
                            counts.failures += 1;
                            let name = test
                                .children()
                                .find(|node| node.has_tag_name("Name"))
                                .and_then(|node| node.text())
                                .unwrap_or("unknown")
                                .to_string();
                            let message = test
                                .descendants()
                                .find(|node| node.has_tag_name("Measurement"))
                                .and_then(|node| {
                                    node.children()
                                        .find(|child| child.has_tag_name("Value"))
                                        .and_then(|value| value.text())
                                })
                                .unwrap_or("CTest failure")
                                .trim()
                                .to_string();
                            failures.push(FailureDetail { name, message });
                        }
                    }
                }
            }
        }
        "testsuites" => {
            for suite in root
                .children()
                .filter(|node| node.has_tag_name("testsuite"))
            {
                counts.tests += attribute_as_u64(&suite, "tests");
                counts.failures += attribute_as_u64(&suite, "failures");
                counts.errors += attribute_as_u64(&suite, "errors");
                counts.skipped += attribute_as_u64(&suite, "skipped");
                failures.extend(extract_testsuite_failures(&suite));
            }
        }
        "testsuite" => {
            counts.tests = attribute_as_u64(&root, "tests");
            counts.failures = attribute_as_u64(&root, "failures");
            counts.errors = attribute_as_u64(&root, "errors");
            counts.skipped = attribute_as_u64(&root, "skipped");
            failures.extend(extract_testsuite_failures(&root));
        }
        tag => {
            return Err(format!("Unsupported JUnit root element '{tag}'").into());
        }
    }

    Ok((counts, failures))
}

fn attribute_as_u64(node: &roxmltree::Node<'_, '_>, name: &str) -> u64 {
    node.attribute(name)
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0)
}

fn extract_testsuite_failures(node: &roxmltree::Node<'_, '_>) -> Vec<FailureDetail> {
    let mut failures = Vec::new();
    for testcase in node
        .children()
        .filter(|child| child.has_tag_name("testcase"))
    {
        let testcase_name = testcase.attribute("name").unwrap_or("unknown");
        let testcase_class = testcase.attribute("classname").unwrap_or_default();
        for failure in testcase
            .children()
            .filter(|child| child.has_tag_name("failure") || child.has_tag_name("error"))
        {
            let name = if testcase_class.is_empty() {
                testcase_name.to_string()
            } else {
                format!("{testcase_class} {testcase_name}")
            };
            let message_attr = failure.attribute("message").unwrap_or_default().trim();
            let body = failure.text().unwrap_or_default().trim();
            let message = if !body.is_empty() {
                body.to_string()
            } else if !message_attr.is_empty() {
                message_attr.to_string()
            } else {
                "Test failure".to_string()
            };
            failures.push(FailureDetail { name, message });
        }
    }
    failures
}

fn collect_result_entries(
    config: &ResultConfig,
) -> Result<Vec<ResultEntry>, Box<dyn std::error::Error>> {
    if !config.result_base.exists() {
        return Err(format!(
            "Test result base does not exist: {}",
            config.result_base.display()
        )
        .into());
    }

    let mut entries = Vec::new();
    for package_dir in fs::read_dir(&config.result_base)? {
        let package_dir = package_dir?;
        if !package_dir.file_type()?.is_dir() {
            continue;
        }
        let package_name = package_dir.file_name().to_string_lossy().to_string();
        let package_path = package_dir.path();

        let xml_paths = find_result_xml_files(&package_path);
        let rc_path = package_path.join("colcon_test.rc");
        if xml_paths.is_empty() && rc_path.is_file() {
            let rc = fs::read_to_string(&rc_path).unwrap_or_default();
            let rc_code = rc.trim().parse::<i32>().unwrap_or(0);
            if rc_code != 0 {
                entries.push(ResultEntry {
                    package_name: package_name.clone(),
                    path: rc_path,
                    status: ResultStatus::Failed,
                    counts: None,
                    detail: Some(format!("colcon_test.rc={rc_code}")),
                    failures: Vec::new(),
                });
            }
        }

        for xml_path in xml_paths {
            match parse_result_xml(&xml_path) {
                Ok((counts, failures)) => {
                    let status = if counts.errors > 0 {
                        ResultStatus::Error
                    } else if counts.failures > 0 {
                        ResultStatus::Failed
                    } else {
                        ResultStatus::Passed
                    };
                    entries.push(ResultEntry {
                        package_name: package_name.clone(),
                        path: xml_path,
                        status,
                        counts: Some(counts),
                        detail: None,
                        failures,
                    });
                }
                Err(error) => {
                    entries.push(ResultEntry {
                        package_name: package_name.clone(),
                        path: xml_path,
                        status: ResultStatus::Error,
                        counts: None,
                        detail: Some(error.to_string()),
                        failures: Vec::new(),
                    });
                }
            }
        }
    }

    entries.sort_by(|left, right| {
        left.package_name
            .cmp(&right.package_name)
            .then_with(|| left.path.cmp(&right.path))
    });
    Ok(entries)
}

fn find_result_xml_files(package_path: &Path) -> Vec<PathBuf> {
    WalkDir::new(package_path)
        .follow_links(true)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| {
            let path = entry.into_path();
            (path.extension().and_then(|ext| ext.to_str()) == Some("xml")).then_some(path)
        })
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name == "pytest.xml" || name == "Test.xml")
                .unwrap_or(false)
                || path
                    .components()
                    .any(|component| component.as_os_str() == "test_results")
        })
        .collect()
}

fn confirm_delete(
    entries: &[ResultEntry],
    assume_yes: bool,
) -> Result<bool, Box<dyn std::error::Error>> {
    if assume_yes {
        return Ok(true);
    }

    print!(
        "Delete {} test result files? [y/N] ",
        entries.len().to_string().bright_white().bold()
    );
    io::stdout().flush()?;

    let mut response = String::new();
    io::stdin().read_line(&mut response)?;
    let normalized = response.trim().to_ascii_lowercase();
    Ok(matches!(normalized.as_str(), "y" | "yes"))
}

fn delete_result_files(entries: &[ResultEntry]) -> Result<(), Box<dyn std::error::Error>> {
    for entry in entries {
        if entry.path.exists() {
            fs::remove_file(&entry.path)?;
        }
    }
    Ok(())
}

fn render_entry(entry: &ResultEntry, verbose: bool) -> Vec<String> {
    let status = match entry.status {
        ResultStatus::Passed => "passed".bright_green().bold(),
        ResultStatus::Failed => "failed".bright_red().bold(),
        ResultStatus::Error => "error".bright_red().bold(),
    };
    let mut lines = Vec::new();
    if verbose {
        let counts = entry
            .counts
            .as_ref()
            .map(|counts| {
                format!(
                    " tests={} failures={} errors={} skipped={}",
                    counts.tests, counts.failures, counts.errors, counts.skipped
                )
            })
            .unwrap_or_default();
        let detail = entry
            .detail
            .as_deref()
            .map(|detail| format!(" detail={detail}"))
            .unwrap_or_default();
        lines.push(format!(
            "{} {} {}{}{}",
            entry.package_name.bright_white().bold(),
            status,
            entry.path.display(),
            counts,
            detail
        ));
        if entry.status != ResultStatus::Passed {
            for failure in &entry.failures {
                lines.push(format!("- {}", failure.name));
                lines.push("  <<< failure message".to_string());
                for line in failure
                    .message
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                {
                    lines.push(format!("    {line}"));
                }
                lines.push("  >>>".to_string());
            }
        }
    } else {
        lines.push(format!(
            "{} {} {}",
            entry.package_name.bright_white().bold(),
            status,
            entry.path.display()
        ));
    }
    lines
}

fn print_summary(entries: &[ResultEntry], verbose: bool) {
    let mut package_totals = BTreeMap::<String, JUnitCounts>::new();
    let mut failed_files = 0usize;
    let mut error_files = 0usize;
    let mut passed_files = 0usize;

    for entry in entries {
        match entry.status {
            ResultStatus::Passed => passed_files += 1,
            ResultStatus::Failed => failed_files += 1,
            ResultStatus::Error => error_files += 1,
        }
        if let Some(counts) = &entry.counts {
            let total = package_totals
                .entry(entry.package_name.clone())
                .or_default();
            total.tests += counts.tests;
            total.failures += counts.failures;
            total.errors += counts.errors;
            total.skipped += counts.skipped;
        }
    }

    println!("{}", "Test result summary".bright_cyan().bold());
    println!(
        "  {} {}",
        entries.len().to_string().bright_white().bold(),
        "result files".bright_cyan()
    );
    println!(
        "  {} passed  {} failed  {} error",
        passed_files.to_string().bright_green().bold(),
        failed_files.to_string().bright_red().bold(),
        error_files.to_string().bright_red().bold()
    );

    if verbose && !package_totals.is_empty() {
        println!("{}", "Package totals".bright_cyan().bold());
        for (package_name, counts) in package_totals {
            println!(
                "  {} tests={} failures={} errors={} skipped={}",
                package_name.bright_white(),
                counts.tests,
                counts.failures,
                counts.errors,
                counts.skipped
            );
        }
    }
}

fn run_command(matches: ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let config = config_from_matches(&matches)?;
    let mut entries = collect_result_entries(&config)?;
    if !config.all {
        entries.retain(ResultEntry::matches_default_filter);
    }

    if entries.is_empty() {
        println!("{}", "No matching test result files found".bright_yellow());
        return Ok(());
    }

    if config.delete {
        if !confirm_delete(&entries, config.delete_yes)? {
            println!("{}", "Deletion cancelled".bright_yellow());
            return Ok(());
        }
        delete_result_files(&entries)?;
        println!(
            "{} {}",
            "Deleted".bright_green().bold(),
            format!("{} test result files", entries.len()).bright_white()
        );
        return Ok(());
    }

    if config.result_files_only {
        for entry in &entries {
            println!("{}", entry.path.display());
        }
        return Ok(());
    }

    print_summary(&entries, config.verbose);
    for entry in &entries {
        for line in render_entry(entry, config.verbose) {
            println!("{line}");
        }
    }

    Ok(())
}

pub fn handle(matches: ArgMatches) {
    handle_boxed_command_result(run_command(matches));
}

#[cfg(test)]
mod tests {
    use super::{
        collect_result_entries, config_from_matches, parse_result_xml, JUnitCounts, ResultConfig,
        ResultEntry, ResultStatus,
    };
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn parse_junit_counts_supports_testsuite_root() {
        let temp = tempdir().unwrap();
        let xml_path = temp.path().join("pytest.xml");
        fs::write(
            &xml_path,
            r#"<testsuite tests="4" failures="1" errors="2" skipped="1"></testsuite>"#,
        )
        .unwrap();

        assert_eq!(
            parse_result_xml(&xml_path).unwrap().0,
            JUnitCounts {
                tests: 4,
                failures: 1,
                errors: 2,
                skipped: 1,
            }
        );
    }

    #[test]
    fn parse_junit_counts_supports_testsuites_root() {
        let temp = tempdir().unwrap();
        let xml_path = temp.path().join("Test.xml");
        fs::write(
            &xml_path,
            r#"<testsuites><testsuite tests="2" failures="1" errors="0" skipped="0"></testsuite><testsuite tests="3" failures="0" errors="1" skipped="2"></testsuite></testsuites>"#,
        )
        .unwrap();

        assert_eq!(
            parse_result_xml(&xml_path).unwrap().0,
            JUnitCounts {
                tests: 5,
                failures: 1,
                errors: 1,
                skipped: 2,
            }
        );
    }

    #[test]
    fn parse_junit_counts_supports_ctest_site_root() {
        let temp = tempdir().unwrap();
        let xml_path = temp.path().join("Test.xml");
        fs::write(
            &xml_path,
            r#"<Site><Testing><Test Status="passed"></Test><Test Status="failed"></Test><Test Status="notrun"></Test></Testing></Site>"#,
        )
        .unwrap();

        assert_eq!(
            parse_result_xml(&xml_path).unwrap().0,
            JUnitCounts {
                tests: 3,
                failures: 1,
                errors: 0,
                skipped: 1,
            }
        );
    }

    #[test]
    fn parse_result_xml_extracts_failure_details() {
        let temp = tempdir().unwrap();
        let xml_path = temp.path().join("pytest.xml");
        fs::write(
            &xml_path,
            r#"<testsuite tests="1" failures="1" errors="0" skipped="0"><testcase classname="demo_pkg.test" name="test_demo"><failure message="assert 1 == 0">line 1
line 2</failure></testcase></testsuite>"#,
        )
        .unwrap();

        let (_, failures) = parse_result_xml(&xml_path).unwrap();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].name, "demo_pkg.test test_demo");
        assert!(failures[0].message.contains("line 1"));
    }

    #[test]
    fn collect_result_entries_reads_xml_and_rc_files() {
        let temp = tempdir().unwrap();
        let build_base = temp.path().join("build");
        let package_dir = build_base.join("demo_pkg");
        fs::create_dir_all(package_dir.join("test_results")).unwrap();
        fs::create_dir_all(package_dir.join("Testing").join("20260307-1051")).unwrap();
        fs::write(package_dir.join("colcon_test.rc"), "1\n").unwrap();
        fs::write(
            package_dir.join("test_results").join("pytest.xml"),
            r#"<testsuite tests="3" failures="1" errors="0" skipped="0"></testsuite>"#,
        )
        .unwrap();
        fs::write(
            package_dir
                .join("Testing")
                .join("20260307-1051")
                .join("Test.xml"),
            r#"<Site><Testing></Testing></Site>"#,
        )
        .unwrap();

        let entries = collect_result_entries(&ResultConfig {
            result_base: build_base,
            all: false,
            verbose: false,
            result_files_only: false,
            delete: false,
            delete_yes: false,
        })
        .unwrap();

        assert_eq!(entries.len(), 2);
        assert!(entries.iter().any(|entry| {
            entry.path.ends_with("Test.xml")
                && entry.status == ResultStatus::Passed
                && entry.counts
                    == Some(JUnitCounts {
                        tests: 0,
                        failures: 0,
                        errors: 0,
                        skipped: 0,
                    })
        }));
        assert!(entries.iter().any(|entry| {
            entry.path.ends_with("pytest.xml")
                && entry.status == ResultStatus::Failed
                && entry.counts
                    == Some(JUnitCounts {
                        tests: 3,
                        failures: 1,
                        errors: 0,
                        skipped: 0,
                    })
        }));
        assert!(!entries
            .iter()
            .any(|entry| entry.path.ends_with("colcon_test.rc")));
    }

    #[test]
    fn collect_result_entries_uses_rc_file_when_no_xml_exists() {
        let temp = tempdir().unwrap();
        let build_base = temp.path().join("build");
        let package_dir = build_base.join("demo_pkg");
        fs::create_dir_all(&package_dir).unwrap();
        fs::write(package_dir.join("colcon_test.rc"), "1\n").unwrap();

        let entries = collect_result_entries(&ResultConfig {
            result_base: build_base,
            all: false,
            verbose: false,
            result_files_only: false,
            delete: false,
            delete_yes: false,
        })
        .unwrap();

        assert_eq!(entries.len(), 1);
        assert!(entries
            .iter()
            .any(|entry| entry.path.ends_with("colcon_test.rc")
                && entry.status == ResultStatus::Failed));
    }

    #[test]
    fn config_from_matches_uses_custom_result_base() {
        let matches = crate::arguments::work::cmd()
            .try_get_matches_from([
                "work",
                "test-result",
                "--test-result-base",
                "out/build-tree",
                "--all",
                "--verbose",
                "--result-files-only",
                "--delete",
                "--delete-yes",
            ])
            .unwrap();
        let (_, submatches) = matches.subcommand().unwrap();

        let config = config_from_matches(submatches).unwrap();
        assert!(config
            .result_base
            .ends_with(PathBuf::from("out/build-tree")));
        assert!(config.all);
        assert!(config.verbose);
        assert!(config.result_files_only);
        assert!(config.delete);
        assert!(config.delete_yes);
    }

    #[test]
    fn default_filter_keeps_only_non_passing_entries() {
        let passed = ResultEntry {
            package_name: "demo".to_string(),
            path: PathBuf::from("/tmp/passed.xml"),
            status: ResultStatus::Passed,
            counts: None,
            detail: None,
            failures: Vec::new(),
        };
        let failed = ResultEntry {
            package_name: "demo".to_string(),
            path: PathBuf::from("/tmp/failed.xml"),
            status: ResultStatus::Failed,
            counts: None,
            detail: None,
            failures: Vec::new(),
        };

        assert!(!passed.matches_default_filter());
        assert!(failed.matches_default_filter());
    }
}
