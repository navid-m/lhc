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
        print_usage(&args[0]);
        process::exit(1);
    }

    let server_path = &args[1];
    let server_args: Vec<String> = args[2..].to_vec();

    eprintln!("Server: {}", server_path);

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

fn print_usage(program: &str) {
    eprintln!(
        r#"
  lhc - LSP Health Checker

  Usage: {} <lsp-server-path> [server-args...]

  Examples:
    {} rust-analyzer
    {} clangd --log=error
    {} zls
    {} pyright-langserver --stdio

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
        program, program, program, program, program
    );
}
