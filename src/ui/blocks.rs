use console::style;

fn pluralize<'a>(count: usize, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 {
        singular
    } else {
        plural
    }
}

pub fn print_section(title: &str) {
    println!("{}", style(title).bold().cyan());
}

pub fn print_total(count: usize, singular: &str, plural: &str) {
    println!(
        "{} {}",
        style("Total:").bold().green(),
        style(format!("{} {}", count, pluralize(count, singular, plural))).bold()
    );
}
