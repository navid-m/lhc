mod checker;
mod display;
mod languages;
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
    let mut server_args: Vec<String> = Vec::new();
    let mut enable_logging = false;
    let mut language: Option<String> = None;
    let mut ref_file: Option<String> = None;

    for arg in args[2..].iter() {
        if arg == "--log" {
            enable_logging = true;
        } else if let Some(lang) = arg.strip_prefix("--language=") {
            language = Some(lang.to_string());
        } else if let Some(ref_path) = arg.strip_prefix("--ref=") {
            ref_file = Some(ref_path.to_string());
        } else {
            server_args.push(arg.clone());
        }
    }

    let log_file_path = if enable_logging {
        let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
        Some(format!("lhc-{}.log", timestamp))
    } else {
        None
    };
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

    let mut health_checker = match HealthChecker::init(server_path, &server_args, log_file_path, language, ref_file) {
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

  Usage: lhc <lsp-server-path> [server-args...] [--log] [--language=<lang>] [--ref=<file>]

  Options:
    --language=<lang>   Use a language-specific sample (rust, c, cpp, python, d, zig,
                        csharp, nim, hare, scheme, java, kotlin, crystal)
    --ref=<file>        Use a custom source file for testing
    --log               Write errors to lhc-TIMESTAMP.log file

  Examples:
    lhc rust-analyzer
    lhc clangd --language=c --log
    lhc liger --language=crystal
    lhc zls --language=zig
    lhc pyright-langserver --stdio --language=python
    lhc clangd --ref=/path/to/test.cpp --log

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
