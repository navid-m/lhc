mod checker;
mod display;
mod languages;
mod lsp;
mod run_bridge;
mod windows;

use checker::HealthChecker;
use run_bridge::run_and_show_diff_checks;
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

        if args[1] == "--list-checks" {
            print_checked_capabilities();
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
    let mut dserver_args: Vec<String> = Vec::new();
    let mut enable_logging = false;
    let mut language: Option<String> = None;
    let mut ref_file: Option<String> = None;
    let mut lsp_flags: Option<String> = None;
    let mut dlsp_flags: Option<String> = None;
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

        if let Some(dlsp_fl) = arg.strip_prefix("--dlsp-flags=") {
            if diff_server == None {
                print_usage();
                process::exit(1);
            }
            dlsp_flags = Some(dlsp_fl.to_string());
        }
    }

    if let Some(flags) = lsp_flags {
        for flag in flags.split_whitespace() {
            server_args.push(flag.to_string());
        }
    }

    if let Some(other_flags) = dlsp_flags {
        for dflag in other_flags.split_whitespace() {
            dserver_args.push(dflag.to_string());
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
        Some(format!(
            "lhc-{}-{}-{}.log",
            clean_server_path,
            language.clone().unwrap(),
            timestamp
        ))
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

    if let std::ops::ControlFlow::Break(_) = run_and_show_diff_checks(
        server_path,
        &language,
        ref_file,
        diff_server,
        &mut health_checker,
        &results,
        dserver_args,
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

fn print_usage() {
    eprintln!(
        r#"lhc - An LSP Server Health Checker

Usage: lhc <lsp-server> [options]

Options:
    --lang=<lang>       Use a language-specific sample (e.g. csharp, c...)
    --ref=<file>        Use a custom source file for testing
    --log               Write errors to lhc-server-timestamp.log file
    --lsp-flags="<f>"   Pass flags to the LSP server
    --dlsp-flags="<f>"  Pass flags to LSP server number two in the diff, must be used with --diff
    --diff=<server>     Compare latency and capabilities against another language server
    --list-langs        List all built-in languages
    --list-checks       List all capability checks that are carried out
    --version           Display the version of lhc
    --json              Output results as JSON
    --help              Show this help message

For example:
    $ lhc clangd --lang=c --diff=ccls --log
    $ lhc liger --lang=crystal
    $ lhc zls --lang=zig
    $ lhc rust-analyzer --lang=rust --lsp-flags="--stdio"
    $ lhc clangd --ref=/path/to/test.cpp --log"#,
    );
}

fn print_checked_capabilities() {
    let mut cap_table = comfy_table::Table::new();

    cap_table.load_preset(comfy_table::presets::UTF8_BORDERS_ONLY);
    cap_table.set_header(comfy_table::Row::from(vec!["Capability", "Name"]));

    for (k, v) in checker::LEFTOVER_CHECKS {
        cap_table.add_row(vec![comfy_table::Cell::new(k), comfy_table::Cell::new(v)]);
    }

    println!("{}", cap_table);
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
