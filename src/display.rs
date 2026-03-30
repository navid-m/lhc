use crate::checker::{CheckResult, CheckStatus};
use crate::lsp::ServerCapabilities;
use comfy_table::modifiers::UTF8_SOLID_INNER_BORDERS;
use comfy_table::presets::UTF8_HORIZONTAL_ONLY;
use comfy_table::{Cell, Color, ContentArrangement, Table};
use serde::Serialize;

const SUMMARY_TABLE_OFFSET: usize = 32;
const ICON_PASS: &str = "✓";
const ICON_FAIL: &str = "✗";
const ICON_SKIP: &str = "○";
const ICON_TIME: &str = "◷";

#[derive(Serialize)]
struct JsonResult {
    server: String,
    language: String,
    healthy: bool,
    summary: Summary,
    checks: Vec<JsonCheckResult>,
}

#[derive(Serialize)]
struct Summary {
    passed: usize,
    failed: usize,
    timed_out: usize,
    skipped: usize,
}

#[derive(Serialize)]
struct JsonCheckResult {
    name: &'static str,
    method: &'static str,
    status: String,
    detail: String,
    duration_ms: i64,
}

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

pub fn render_table(
    results: &[CheckResult],
    server_path: String,
    language: String,
    json_output: bool,
) {
    if json_output {
        render_json(results, server_path, language);
        return;
    }

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

    summary_table.load_preset(UTF8_SOLID_INNER_BORDERS);
    summary_table.add_row(vec![
        Cell::new(passed),
        Cell::new(timed_out),
        Cell::new(failed),
        Cell::new(skipped),
    ]);

    summary_table.column_mut(3).unwrap().set_constraint(
        comfy_table::ColumnConstraint::Boundaries {
            lower: comfy_table::Width::Fixed(
                (table_width.saturating_sub(SUMMARY_TABLE_OFFSET)) as u16,
            ),
            upper: comfy_table::Width::Fixed(
                (table_width.saturating_sub(SUMMARY_TABLE_OFFSET)) as u16,
            ),
        },
    );

    let mut health_box = Table::new();

    health_box.load_preset(comfy_table::presets::UTF8_HORIZONTAL_ONLY);

    if failed == 0 && timed_out == 0 {
        health_box.add_row(vec![Cell::new("Server is healthy")]);
    } else {
        health_box.add_row(vec![Cell::new(format!(
            "Server {} has issues with the {} language.",
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

fn render_json(results: &[CheckResult], server_path: String, language: String) {
    let mut passed: usize = 0;
    let mut failed: usize = 0;
    let mut skipped: usize = 0;
    let mut timed_out: usize = 0;

    let checks: Vec<JsonCheckResult> = results
        .iter()
        .map(|r| {
            match r.status {
                CheckStatus::Passed => passed += 1,
                CheckStatus::Failed => failed += 1,
                CheckStatus::Skipped => skipped += 1,
                CheckStatus::Timeout => timed_out += 1,
            }
            JsonCheckResult {
                name: r.name,
                method: r.method,
                status: r.status.as_str().to_string(),
                detail: r.detail.clone(),
                duration_ms: r.duration_ms,
            }
        })
        .collect();

    let healthy = failed == 0 && timed_out == 0;

    let json_result = JsonResult {
        server: server_path,
        language,
        healthy,
        summary: Summary {
            passed,
            failed,
            timed_out,
            skipped,
        },
        checks,
    };

    println!("{}", serde_json::to_string_pretty(&json_result).unwrap());
}

const ICON_ONLY_A: &str = "◀";
const ICON_ONLY_B: &str = "▶";
const ICON_BOTH: &str = "=";
const ICON_NEITHER: &str = "—";

struct CapabilityEntry {
    label: &'static str,
    a: bool,
    b: bool,
}

fn capability_entries(a: &ServerCapabilities, b: &ServerCapabilities) -> Vec<CapabilityEntry> {
    vec![
        CapabilityEntry {
            label: "Hover",
            a: a.hover_provider,
            b: b.hover_provider,
        },
        CapabilityEntry {
            label: "Signature Help",
            a: a.signature_help_provider,
            b: b.signature_help_provider,
        },
        CapabilityEntry {
            label: "Completion",
            a: a.completion_provider,
            b: b.completion_provider,
        },
        CapabilityEntry {
            label: "Go to Definition",
            a: a.definition_provider,
            b: b.definition_provider,
        },
        CapabilityEntry {
            label: "Type Definition",
            a: a.type_definition_provider,
            b: b.type_definition_provider,
        },
        CapabilityEntry {
            label: "Implementation",
            a: a.implementation_provider,
            b: b.implementation_provider,
        },
        CapabilityEntry {
            label: "Find References",
            a: a.references_provider,
            b: b.references_provider,
        },
        CapabilityEntry {
            label: "Document Symbols",
            a: a.document_symbol_provider,
            b: b.document_symbol_provider,
        },
        CapabilityEntry {
            label: "Workspace Symbols",
            a: a.workspace_symbol_provider,
            b: b.workspace_symbol_provider,
        },
        CapabilityEntry {
            label: "Formatting",
            a: a.document_formatting_provider,
            b: b.document_formatting_provider,
        },
        CapabilityEntry {
            label: "Code Actions",
            a: a.code_action_provider,
            b: b.code_action_provider,
        },
        CapabilityEntry {
            label: "Rename Symbol",
            a: a.rename_provider,
            b: b.rename_provider,
        },
        CapabilityEntry {
            label: "Prepare Rename",
            a: a.prepare_rename_provider,
            b: b.prepare_rename_provider,
        },
        CapabilityEntry {
            label: "Inlay Hints",
            a: a.inlay_hint_provider,
            b: b.inlay_hint_provider,
        },
        CapabilityEntry {
            label: "Code Lens",
            a: a.code_lens_provider,
            b: b.code_lens_provider,
        },
        CapabilityEntry {
            label: "Semantic Tokens",
            a: a.semantic_tokens_provider,
            b: b.semantic_tokens_provider,
        },
        CapabilityEntry {
            label: "Folding Range",
            a: a.folding_range_provider,
            b: b.folding_range_provider,
        },
        CapabilityEntry {
            label: "Linked Editing Range",
            a: a.linked_editing_range_provider,
            b: b.linked_editing_range_provider,
        },
        CapabilityEntry {
            label: "Selection Range",
            a: a.selection_range_provider,
            b: b.selection_range_provider,
        },
        CapabilityEntry {
            label: "Document Highlight",
            a: a.document_highlight_provider,
            b: b.document_highlight_provider,
        },
        CapabilityEntry {
            label: "Publish Diagnostics",
            a: a.publish_diagnostics_provider,
            b: b.publish_diagnostics_provider,
        },
        CapabilityEntry {
            label: "Execute Command",
            a: a.execute_command_provider,
            b: b.execute_command_provider,
        },
        CapabilityEntry {
            label: "Did Change Configuration",
            a: a.did_change_configuration_provider,
            b: b.did_change_configuration_provider,
        },
        CapabilityEntry {
            label: "Did Change Workspace Folders",
            a: a.did_change_workspace_folders_provider,
            b: b.did_change_workspace_folders_provider,
        },
    ]
}

/// Render a side-by-side diff of two LSP servers' capabilities and latencies.
pub fn render_diff(
    server_a: &str,
    results_a: &[CheckResult],
    caps_a: &ServerCapabilities,
    server_b: &str,
    results_b: &[CheckResult],
    caps_b: &ServerCapabilities,
    language: &str,
) {
    let mut header_box = Table::new();
    header_box.load_preset(UTF8_HORIZONTAL_ONLY);
    header_box.add_row(vec![Cell::new(format!(
        "Diff: {} vs {}",
        server_a, server_b
    ))]);
    header_box.add_row(vec![Cell::new(format!("Language: {}", language))]);
    println!("{}", header_box);
    println!();

    let mut lat_table = Table::new();
    lat_table
        .load_preset(UTF8_HORIZONTAL_ONLY)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Check").fg(Color::Cyan),
            Cell::new("LSP Method").fg(Color::Cyan),
            Cell::new(format!("{} status", server_a)).fg(Color::Cyan),
            Cell::new(format!("{} ms", server_a)).fg(Color::Cyan),
            Cell::new(format!("{} status", server_b)).fg(Color::Cyan),
            Cell::new(format!("{} ms", server_b)).fg(Color::Cyan),
            Cell::new("diff ms").fg(Color::Cyan),
        ]);

    let lookup_b: std::collections::HashMap<&str, &CheckResult> =
        results_b.iter().map(|r| (r.method, r)).collect();

    for ra in results_a {
        let rb_opt = lookup_b.get(ra.method).copied();

        let (icon_a, color_a) = status_icon_color(ra.status);
        let lat_a = latency_display(ra);

        let (icon_b, color_b, lat_b) = if let Some(rb) = rb_opt {
            let (i, c) = status_icon_color(rb.status);
            (i, c, latency_display(rb))
        } else {
            (ICON_SKIP, Color::DarkGrey, "—".to_string())
        };

        let delta_cell = match (ra.status, rb_opt.map(|r| r.status)) {
            (CheckStatus::Passed, Some(CheckStatus::Passed)) => {
                let da = ra.duration_ms;
                let db = rb_opt.unwrap().duration_ms;
                let delta = da - db;
                let text = if delta == 0 {
                    "0 ms".to_string()
                } else if delta > 0 {
                    format!("+{} ms", delta)
                } else {
                    format!("{} ms", delta)
                };
                let color = if delta > 0 {
                    Color::Red
                } else if delta < 0 {
                    Color::Green
                } else {
                    Color::White
                };
                Cell::new(text).fg(color)
            }
            _ => Cell::new("—").fg(Color::DarkGrey),
        };

        lat_table.add_row(vec![
            Cell::new(ra.name),
            Cell::new(ra.method),
            Cell::new(format!("{} {}", icon_a, ra.status.as_str())).fg(color_a),
            Cell::new(&lat_a).fg(color_a),
            Cell::new(format!(
                "{} {}",
                icon_b,
                rb_opt.map(|r| r.status.as_str()).unwrap_or("—")
            ))
            .fg(color_b),
            Cell::new(&lat_b).fg(color_b),
            delta_cell,
        ]);
    }

    println!("{}", lat_table);
    println!();

    let entries = capability_entries(caps_a, caps_b);
    let has_diff = entries.iter().any(|e| e.a != e.b);
    let mut cap_table = Table::new();

    cap_table
        .load_preset(UTF8_HORIZONTAL_ONLY)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Capability").fg(Color::Cyan),
            Cell::new(server_a).fg(Color::Cyan),
            Cell::new(server_b).fg(Color::Cyan),
            Cell::new("Note").fg(Color::Cyan),
        ]);

    for e in &entries {
        let (a_icon, a_color) = if e.a {
            (ICON_PASS, Color::Green)
        } else {
            (ICON_FAIL, Color::DarkGrey)
        };
        let (b_icon, b_color) = if e.b {
            (ICON_PASS, Color::Green)
        } else {
            (ICON_FAIL, Color::DarkGrey)
        };

        let (note, note_color) = match (e.a, e.b) {
            (true, true) => (format!("{} both advertise", ICON_BOTH), Color::White),
            (false, false) => (
                format!("{} neither advertises", ICON_NEITHER),
                Color::DarkGrey,
            ),
            (true, false) => (format!("{} only {}", ICON_ONLY_A, server_a), Color::Yellow),
            (false, true) => (format!("{} only {}", ICON_ONLY_B, server_b), Color::Yellow),
        };

        cap_table.add_row(vec![
            Cell::new(e.label),
            Cell::new(a_icon).fg(a_color),
            Cell::new(b_icon).fg(b_color),
            Cell::new(note).fg(note_color),
        ]);
    }

    println!("{}", cap_table);
    println!();

    let mut summary_box = Table::new();
    summary_box.load_preset(UTF8_HORIZONTAL_ONLY);
    if has_diff {
        let only_a: Vec<&str> = entries
            .iter()
            .filter(|e| e.a && !e.b)
            .map(|e| e.label)
            .collect();
        let only_b: Vec<&str> = entries
            .iter()
            .filter(|e| e.b && !e.a)
            .map(|e| e.label)
            .collect();
        if !only_a.is_empty() {
            summary_box.add_row(vec![
                Cell::new(format!(
                    "{} advertises but {} does not: {}",
                    server_a,
                    server_b,
                    only_a.join(", ")
                ))
                .fg(Color::Yellow),
            ]);
        }
        if !only_b.is_empty() {
            summary_box.add_row(vec![
                Cell::new(format!(
                    "{} advertises but {} does not: {}",
                    server_b,
                    server_a,
                    only_b.join(", ")
                ))
                .fg(Color::Yellow),
            ]);
        }
    } else {
        summary_box.add_row(vec![Cell::new(
            "Both servers advertise the same capabilities.",
        )]);
    }
    println!("{}", summary_box);
}

fn status_icon_color(status: CheckStatus) -> (&'static str, Color) {
    match status {
        CheckStatus::Passed => (ICON_PASS, Color::Green),
        CheckStatus::Failed => (ICON_FAIL, Color::Red),
        CheckStatus::Skipped => (ICON_SKIP, Color::DarkGrey),
        CheckStatus::Timeout => (ICON_TIME, Color::Magenta),
    }
}

fn latency_display(r: &CheckResult) -> String {
    if r.status == CheckStatus::Skipped || r.duration_ms == 0 {
        "—".to_string()
    } else {
        format!("{} ms", r.duration_ms)
    }
}
