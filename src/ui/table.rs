use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Attribute, Cell, ContentArrangement, Table,
};
use console::Term;

pub fn print_table(headers: &[&str], rows: Vec<Vec<String>>) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);

    let (_, width) = Term::stdout().size();
    if width > 0 {
        table.set_width(width);
    }

    table.set_header(
        headers
            .iter()
            .map(|header| Cell::new(*header).add_attribute(Attribute::Bold))
            .collect::<Vec<_>>(),
    );

    for row in rows {
        table.add_row(row.into_iter().map(Cell::new).collect::<Vec<_>>());
    }

    println!("{table}");
}
