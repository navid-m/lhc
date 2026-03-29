use crate::checker::{CheckResult, CheckStatus};
use comfy_table::presets::UTF8_HORIZONTAL_ONLY;
use comfy_table::{Cell, Color, ContentArrangement, Table};

const ICON_PASS: &str = "✓";
const ICON_FAIL: &str = "✗";
const ICON_SKIP: &str = "○";
const ICON_TIME: &str = "◌";

pub fn render_table(results: &[CheckResult]) {
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
            CheckStatus::Skipped => (ICON_SKIP, Color::Yellow),
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

    println!();
    println!("{}", table);
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

    let mut health_box = Table::new();
    health_box.load_preset(comfy_table::presets::UTF8_BORDERS_ONLY);
    if failed == 0 && timed_out == 0 {
        health_box.add_row(vec![Cell::new("[ok] Server is healthy")]);
    } else {
        health_box.add_row(vec![Cell::new("[!] Server has issues")]);
        health_box.add_row(vec![Cell::new(summary_table.to_string())]);
    }

    println!("{}", health_box);
}
