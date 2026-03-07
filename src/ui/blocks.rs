use console::style;

fn pluralize<'a>(count: usize, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 {
        singular
    } else {
        plural
    }
}

fn emit_line(line: impl std::fmt::Display, to_stderr: bool) {
    if to_stderr {
        eprintln!("{line}");
    } else {
        println!("{line}");
    }
}

fn emit_section(title: &str, to_stderr: bool) {
    emit_line(style(title).bold().cyan(), to_stderr);
}

fn emit_field(label: &str, value: impl std::fmt::Display, to_stderr: bool) {
    emit_line(
        format!(
            "{} {}",
            style(format!("{label:<18}")).bold().yellow(),
            value
        ),
        to_stderr,
    );
}

fn emit_message(colorized: impl std::fmt::Display, message: &str, to_stderr: bool) {
    emit_line(format!("{colorized} {message}"), to_stderr);
}

fn emit_status(label: &str, fields: &[(&str, String)], to_stderr: bool) {
    let mut rendered = String::new();
    for (index, (name, value)) in fields.iter().enumerate() {
        if index > 0 {
            rendered.push_str("  ");
        }
        rendered.push_str(&format!(
            "{} {}",
            style(format!("{name}:")).bold().yellow(),
            value
        ));
    }

    emit_line(
        format!(
            "{} {}",
            style(format!("{label:<12}")).bold().cyan(),
            rendered
        ),
        to_stderr,
    );
}

pub fn print_section(title: &str) {
    emit_section(title, false);
}

pub fn eprint_section(title: &str) {
    emit_section(title, true);
}

pub fn print_field(label: &str, value: impl std::fmt::Display) {
    emit_field(label, value, false);
}

pub fn eprint_field(label: &str, value: impl std::fmt::Display) {
    emit_field(label, value, true);
}

pub fn print_total(count: usize, singular: &str, plural: &str) {
    emit_line(
        format!(
            "{} {}",
            style("Total:").bold().green(),
            style(format!("{} {}", count, pluralize(count, singular, plural))).bold()
        ),
        false,
    );
}

pub fn print_note(message: &str) {
    emit_message(style("Note:").bold().blue(), message, false);
}

pub fn eprint_note(message: &str) {
    emit_message(style("Note:").bold().blue(), message, true);
}

pub fn eprint_warning(message: &str) {
    emit_message(style("Warning:").bold().yellow(), message, true);
}

pub fn print_success(message: &str) {
    emit_message(style("Done:").bold().green(), message, false);
}

pub fn print_status(label: &str, fields: &[(&str, String)]) {
    emit_status(label, fields, false);
}

pub fn eprint_status(label: &str, fields: &[(&str, String)]) {
    emit_status(label, fields, true);
}
