use crate::checker::{CheckResult, CheckStatus};
use comfy_table::presets::UTF8_HORIZONTAL_ONLY;
use comfy_table::{Cell, Color, ContentArrangement, Table};

const ICON_PASS: &str = "✓";
const ICON_FAIL: &str = "✗";
const ICON_SKIP: &str = "○";
const ICON_TIME: &str = "◌";

pub fn render_header(server_path: &str, language: &str, table_width: usize) {
    let mut starter_box = Table::new();
    starter_box.load_preset(UTF8_HORIZONTAL_ONLY);

    starter_box.add_row(vec![Cell::new(format!("Server: {}", server_path))]);
    starter_box.add_row(vec![Cell::new(format!("Language: {}", language))]);

    starter_box
        .column_mut(0)
        .unwrap()
        .set_constraint(comfy_table::ColumnConstraint::Boundaries {
            lower: comfy_table::Width::Fixed(table_width as u16),
            upper: comfy_table::Width::Fixed(table_width as u16),
        });

    println!("{}", starter_box);
}

pub fn render_table(results: &[CheckResult], server_path: String, language: String) {
    let mut passed: usize = 0;
    let mut failed: usize = 0;
    let mut skipped: usize = 0;
    let mut timed_out: usize = 0;

    for r in results {
        match r.status {
            CheckStatus::Passed => passed += 1,
            CheckStatus::Failed => failed += 1,
            CheckStatus::Skipped => skipped += 1,
            CheckStatus::Timeout => timed_out += 1,
        }
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_HORIZONTAL_ONLY)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Status").fg(Color::Cyan),
            Cell::new("Check").fg(Color::Cyan),
            Cell::new("LSP Method").fg(Color::Cyan),
            Cell::new("Detail").fg(Color::Cyan),
            Cell::new("Latency").fg(Color::Cyan),
        ]);

    for r in results {
        let (icon, color) = match r.status {
            CheckStatus::Passed => (ICON_PASS, Color::Green),
            CheckStatus::Failed => (ICON_FAIL, Color::Red),
            CheckStatus::Skipped => (ICON_SKIP, Color::DarkGrey),
            CheckStatus::Timeout => (ICON_TIME, Color::Magenta),
        };

        let status_str = format!("{} {}", icon, r.status.as_str());

        let latency_display = if r.status == CheckStatus::Skipped || r.duration_ms == 0 {
            "—".to_string()
        } else {
            format!("{} ms", r.duration_ms)
        };

        table.add_row(vec![
            Cell::new(status_str).fg(color),
            Cell::new(r.name),
            Cell::new(r.method),
            Cell::new(r.detail.as_str()),
            Cell::new(latency_display).fg(color),
        ]);
    }

    let table_str = table.to_string();
    let table_width = table_str
        .lines()
        .next()
        .map(|line| line.chars().count())
        .unwrap_or(132);

    render_header(&server_path, &language, table_width);

    println!();
    println!("{}", table_str);
    println!();

    let mut summary_table = Table::new();
    summary_table.set_header(vec![
        Cell::new("Passed"),
        Cell::new("Failed"),
        Cell::new("Timed out"),
        Cell::new("Skipped"),
    ]);

    summary_table.load_preset(UTF8_HORIZONTAL_ONLY);
    summary_table.add_row(vec![
        Cell::new(passed),
        Cell::new(timed_out),
        Cell::new(failed),
        Cell::new(skipped),
    ]);

    summary_table.column_mut(3).unwrap().set_constraint(
        comfy_table::ColumnConstraint::LowerBoundary(comfy_table::Width::Percentage(30)),
    );

    let mut health_box = Table::new();

    health_box.load_preset(comfy_table::presets::UTF8_HORIZONTAL_ONLY);

    if failed == 0 && timed_out == 0 {
        health_box.add_row(vec![Cell::new("Server is healthy")]);
    } else {
        health_box.add_row(vec![Cell::new(format!(
            "Server {} has issues with language {}.",
            server_path, language
        ))]);
        health_box.add_row(vec![Cell::new(summary_table.to_string())]);
    }

    health_box
        .column_mut(0)
        .unwrap()
        .set_constraint(comfy_table::ColumnConstraint::Boundaries {
            lower: comfy_table::Width::Fixed(table_width as u16),
            upper: comfy_table::Width::Fixed(table_width as u16),
        });

    println!("{}", health_box);
}
