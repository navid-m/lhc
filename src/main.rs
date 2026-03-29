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

    if args.len() == 2 && args[1] == "--list-langs" {
        print_supported_languages();
        process::exit(0);
    }

    let server_path = &args[1];
    let mut server_args: Vec<String> = Vec::new();
    let mut enable_logging = false;
    let mut language: Option<String> = None;
    let mut ref_file: Option<String> = None;
    let mut lsp_flags: Option<String> = None;
    let mut seen_required = false;

    for arg in args[2..].iter() {
        if arg == "--log" {
            enable_logging = true;
        } else if arg == "--list-langs" {
            print_supported_languages();
            process::exit(0);
        } else if let Some(lang) = arg.strip_prefix("--lang=") {
            language = Some(lang.to_string());
            seen_required = true;
        } else if let Some(ref_path) = arg.strip_prefix("--ref=") {
            ref_file = Some(ref_path.to_string());
            seen_required = true;
        } else if let Some(flags) = arg.strip_prefix("--lsp-flags=") {
            lsp_flags = Some(flags.to_string());
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
        ref_file,
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

    health_checker.deinit();

    display::render_table(&results, server_path.clone(), language.unwrap());
}

fn print_usage() {
    eprintln!(
        r#"lhc - LSP Server Health Checker

Usage: lhc <lsp-server> [--log] [--lang=<lang>] [--ref=<file>] [--lsp-flags="<flags>"] [--list-langs]

Options:
    --lang=<lang>       Use a language-specific sample (e.g. rust, c, cpp, etc...)
    --ref=<file>        Use a custom source file for testing
    --log               Write errors to lhc-<timestamp>.log file
    --lsp-flags="<f>"   Pass flags to the LSP server
    --list-langs        List all built-in languages

For example:
    lhc clangd --lang=c --log
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

    println!("\nUse --lang=<language> to test with a specific language.");
}
