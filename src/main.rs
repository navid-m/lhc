mod checker;
mod display;
mod lsp;
mod windows;

use checker::HealthChecker;
use std::env;
use std::process;

fn main() {
    if let Err(e) = windows::set_console_output_cp_utf8() {
        eprintln!("Warning: Failed to set console code page to UTF-8: {}", e);
    }

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let server_path = &args[1];
    let server_args: Vec<String> = args[2..].to_vec();
    let mut starter_box = comfy_table::Table::new();
    starter_box.load_preset(comfy_table::presets::UTF8_HORIZONTAL_ONLY);
    starter_box.add_row(vec![comfy_table::Cell::new(format!(
        "Server: {}",
        server_path
    ))]);
    starter_box.column_mut(0).unwrap().set_constraint(
        comfy_table::ColumnConstraint::LowerBoundary(comfy_table::Width::Percentage(46)),
    );
    eprintln!("{}", starter_box.to_string());

    let mut health_checker = match HealthChecker::init(server_path, &server_args) {
        Ok(checker) => checker,
        Err(e) => {
            eprintln!("Failed to initialize health checker: {}", e);
            process::exit(1);
        }
    };

    let results = match health_checker.run_all_checks() {
        Ok(results) => results,
        Err(e) => {
            eprintln!("Failed to run health checks: {}", e);
            process::exit(1);
        }
    };

    health_checker.deinit();

    display::render_table(&results);
}

fn print_usage() {
    eprintln!(
        r#"
  lhc - LSP Health Checker

  Usage: lhc <lsp-server-path> [server-args...]

  Examples:
    lhc rust-analyzer
    lhc clangd --log=error
    lhc liger
    lhc zls
    lhc pyright-langserver --stdio

  Checks performed:
    · initialize / initialized handshake
    · textDocument/didOpen
    · textDocument/hover
    · textDocument/signatureHelp
    · textDocument/completion
    · textDocument/definition
    · textDocument/references
    · textDocument/documentSymbol
    · textDocument/formatting
    · textDocument/codeAction
    · textDocument/rename
    · textDocument/inlayHint
    · workspace/symbol
    · shutdown / exit
"#,
    );
}
