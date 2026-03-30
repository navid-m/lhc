mod checker;
mod display;
mod languages;
mod lsp;
mod windows;

use checker::HealthChecker;
use std::collections::HashSet;
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

    if args.len() == 2 {
        if args[1] == "--help" || args[1] == "-h" {
            print_usage();
            process::exit(0);
        }

        if args[1] == "--list-langs" {
            print_supported_languages();
            process::exit(0);
        }

        if args[1] == "--version" || args[1] == "-v" {
            println!(
                "v{}\nBy Navid M (C) - GPL-3.0-only",
                env!("CARGO_PKG_VERSION")
            );
            process::exit(0);
        }
    }

    let server_path = &args[1];
    let mut server_args: Vec<String> = Vec::new();
    let mut enable_logging = false;
    let mut language: Option<String> = None;
    let mut ref_file: Option<String> = None;
    let mut lsp_flags: Option<String> = None;
    let mut json_output = false;
    let mut seen_required = false;
    let mut diff_server: Option<String> = None;

    for arg in args[2..].iter() {
        if arg == "--log" {
            enable_logging = true;
        } else if arg == "--json" {
            json_output = true;
        } else if let Some(lang) = arg.strip_prefix("--lang=") {
            language = Some(lang.to_string());
            seen_required = true;
        } else if let Some(ref_path) = arg.strip_prefix("--ref=") {
            ref_file = Some(ref_path.to_string());
            seen_required = true;
        } else if let Some(flags) = arg.strip_prefix("--lsp-flags=") {
            lsp_flags = Some(flags.to_string());
        } else if let Some(diff) = arg.strip_prefix("--diff=") {
            diff_server = Some(diff.to_string());
        }
    }

    if let Some(flags) = lsp_flags {
        for flag in flags.split_whitespace() {
            server_args.push(flag.to_string());
        }
    }

    if !seen_required {
        eprintln!("Either the --lang=... or --ref=... flags are required.");
        return;
    }

    let server_path_raw = std::path::Path::new(server_path);
    let file_stem = server_path_raw.file_stem().and_then(|s| s.to_str());
    let clean_server_path: String;

    match file_stem {
        Some(_) => clean_server_path = file_stem.unwrap().to_string(),
        None => {
            eprintln!("Error: Couldn't extract stem from server path");
            return;
        }
    }

    let log_file_path = if enable_logging {
        let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
        Some(format!("lhc-{}-{}.log", clean_server_path, timestamp))
    } else {
        None
    };

    let mut health_checker = match HealthChecker::init(
        server_path,
        &server_args,
        log_file_path,
        language.clone(),
        ref_file.clone(),
    ) {
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

    if let std::ops::ControlFlow::Break(_) = run_diff_checks(
        server_path,
        &language,
        ref_file,
        diff_server,
        &mut health_checker,
        &results,
    ) {
        return;
    }

    health_checker.deinit();

    display::render_table(
        &results,
        server_path.clone(),
        language.unwrap(),
        json_output,
    );
}

fn run_diff_checks(
    server_path: &String,
    language: &Option<String>,
    ref_file: Option<String>,
    diff_server: Option<String>,
    health_checker: &mut HealthChecker,
    results: &Vec<checker::CheckResult>,
) -> std::ops::ControlFlow<()> {
    if let Some(ref diff_path) = diff_server {
        let caps_a = health_checker.get_capabilities().clone();
        health_checker.deinit();

        let mut checker_b =
            match HealthChecker::init(diff_path, &[], None, language.clone(), ref_file) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to initialize diff server '{}': {}", diff_path, e);
                    process::exit(1);
                }
            };

        let caps_b = checker_b.get_capabilities().clone();
        let results_b = match checker_b.run_all_checks() {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to run checks on diff server: {}", e);
                eprintln!("Continuing with partial results...");

                let mut partial_results = checker_b.results.clone();
                use checker::CheckResult;
                use checker::CheckStatus;
                let remaining_checks = [
                    ("Hover", "textDocument/hover"),
                    ("Signature Help", "textDocument/signatureHelp"),
                    ("Completion", "textDocument/completion"),
                    ("Go to Definition", "textDocument/definition"),
                    ("Type Definition", "textDocument/typeDefinition"),
                    ("Implementation", "textDocument/implementation"),
                    ("Find References", "textDocument/references"),
                    ("Document Symbols", "textDocument/documentSymbol"),
                    ("Workspace Symbols", "workspace/symbol"),
                    ("Formatting", "textDocument/formatting"),
                    ("Code Action", "textDocument/codeAction"),
                    ("Rename", "textDocument/rename"),
                    ("Prepare Rename", "textDocument/prepareRename"),
                    ("Inlay Hint", "textDocument/inlayHint"),
                    ("Code Lens", "textDocument/codeLens"),
                    ("Semantic Tokens", "textDocument/semanticTokens/full"),
                    ("Folding Range", "textDocument/foldingRange"),
                    ("Linked Editing Range", "textDocument/linkedEditingRange"),
                    ("Selection Range", "textDocument/selectionRange"),
                    ("Document Highlight", "textDocument/documentHighlight"),
                    ("DidChangeConfiguration", "workspace/didChangeConfiguration"),
                    (
                        "DidChangeWorkspaceFolders",
                        "workspace/didChangeWorkspaceFolders",
                    ),
                    ("Execute Command", "workspace/executeCommand"),
                    ("Shutdown", "shutdown"),
                ];

                let existing_names: HashSet<_> = partial_results.iter().map(|r| r.name).collect();

                for (name, method) in &remaining_checks {
                    if !existing_names.contains(name) {
                        partial_results.push(CheckResult {
                            name,
                            method,
                            status: CheckStatus::Failed,
                            detail: "server crashed".to_string(),
                            duration_ms: 0,
                        });
                    }
                }

                checker_b.deinit();
                let lang = language.as_deref().unwrap_or("unknown");
                display::render_diff(
                    server_path,
                    results,
                    &caps_a,
                    diff_path,
                    &partial_results,
                    &caps_b,
                    &lang,
                );
                return std::ops::ControlFlow::Break(());
            }
        };

        let caps_b = checker_b.get_capabilities().clone();
        checker_b.deinit();

        display::render_diff(
            server_path,
            results,
            &caps_a,
            diff_path,
            &results_b,
            &caps_b,
            language.as_deref().unwrap_or("unknown"),
        );
        return std::ops::ControlFlow::Break(());
    }
    std::ops::ControlFlow::Continue(())
}

fn print_usage() {
    eprintln!(
        r#"lhc - LSP Server Health Checker

Usage: lhc <lsp-server> [--log] [--lang=<lang>] [--ref=<file>] [--lsp-flags="<flags>"] [--diff=<server>] [--list-langs]

Options:
    --lang=<lang>       Use a language-specific sample (e.g. rust, c, cpp, etc...)
    --ref=<file>        Use a custom source file for testing
    --log               Write errors to lhc-server-timestamp.log file
    --lsp-flags="<f>"   Pass flags to the LSP server
    --diff=<server>     Compare latency and capabilities against another language server
    --list-langs        List all built-in languages
    --version           Display the version of lhc
    --json              Output results as JSON
    --help              Show this help message

For example:
    lhc clangd --lang=c --diff=ccls --log
    lhc liger --lang=crystal
    lhc zls --lang=zig
    lhc rust-analyzer --lang=rust --lsp-flags="--stdio"
    lhc clangd --ref=/path/to/test.cpp --log"#,
    );
}

fn print_supported_languages() {
    let languages = languages::list_supported_languages();
    println!("Builtin Languages ({}):\n", languages.len());

    for (i, lang) in languages.iter().enumerate() {
        if (i + 1) % 4 == 0 {
            println!("{}", lang);
        } else {
            print!("{:<20}", lang);
        }
    }

    if languages.len() % 4 != 0 {
        println!();
    }

    println!("\nUse --lang=<language> to test with a specific builtin language.");
}
