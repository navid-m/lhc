use crate::checker::{HealthChecker, LEFTOVER_CHECKS};
use std::collections::HashSet;
use std::process;

pub fn run_and_show_diff_checks(
    server_path: &String,
    language: &Option<String>,
    ref_file: Option<String>,
    diff_server: Option<String>,
    health_checker: &mut HealthChecker,
    results: &Vec<crate::checker::CheckResult>,
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
            Err(_) => {
                let mut partial_results = checker_b.results.clone();
                let existing_names: HashSet<_> = partial_results.iter().map(|r| r.name).collect();

                for (name, method) in &LEFTOVER_CHECKS {
                    if !existing_names.contains(name) {
                        partial_results.push(crate::checker::CheckResult {
                            name,
                            method,
                            status: crate::checker::CheckStatus::Failed,
                            detail: "server crashed".to_string(),
                            duration_ms: 0,
                        });
                    }
                }

                checker_b.deinit();
                let lang = language.as_deref().unwrap_or("unknown");
                crate::display::render_diff(
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

        crate::display::render_diff(
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
