use crate::languages::LanguageSample;
use crate::lsp::{Client, ServerCapabilities};
use serde_json::{json, Value};
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::time::{Duration, Instant};

pub const TIMEOUT_MS: u64 = 5000;

fn extract_error_message(resp: &Value) -> String {
    if let Some(error) = resp.get("error") {
        if let Some(obj) = error.as_object() {
            if let Some(message) = obj.get("message").and_then(|m| m.as_str()) {
                return message.to_string();
            }
            if let Some(data) = obj.get("data") {
                if let Some(data_str) = data.as_str() {
                    return data_str.to_string();
                }
                return data.to_string();
            }
            return error.to_string();
        }
    }
    "unknown error".to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckStatus {
    Passed,
    Failed,
    Skipped,
    Timeout,
}

impl CheckStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            CheckStatus::Passed => "passed",
            CheckStatus::Failed => "failed",
            CheckStatus::Skipped => "skipped",
            CheckStatus::Timeout => "timeout",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub name: &'static str,
    pub method: &'static str,
    pub status: CheckStatus,
    pub detail: String,
    pub duration_ms: i64,
}

pub struct HealthChecker {
    client: Client,
    capabilities: ServerCapabilities,
    results: Vec<CheckResult>,
    log_writer: Option<BufWriter<File>>,
    server_name: String,
    sample: LanguageSample,
}

impl HealthChecker {
    pub fn init(
        server_path: &str,
        server_args: &[String],
        log_file_path: Option<String>,
        language: Option<String>,
        ref_file: Option<String>,
    ) -> std::io::Result<Self> {
        let client = Client::init(server_path, server_args)?;

        let log_writer = if let Some(path) = log_file_path {
            let file = File::create(&path)?;
            Some(BufWriter::new(file))
        } else {
            None
        };

        let server_name = std::path::Path::new(server_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(server_path)
            .to_string();

        let sample = if let Some(ref_path) = ref_file {
            let mut file = File::open(&ref_path)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;

            let ext = std::path::Path::new(&ref_path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("txt")
                .to_string();

            let language_id = match ext.as_str() {
                "rs" => "rust".to_string(),
                "c" => "c".to_string(),
                "cpp" | "cc" | "cxx" | "h" | "hpp" => "cpp".to_string(),
                "py" => "python".to_string(),
                "d" => "d".to_string(),
                "zig" => "zig".to_string(),
                "cs" => "csharp".to_string(),
                "nim" => "nim".to_string(),
                "ha" => "hare".to_string(),
                "scm" | "ss" => "scheme".to_string(),
                "java" => "java".to_string(),
                "kt" | "kts" => "kotlin".to_string(),
                "cr" => "crystal".to_string(),
                _ => ext.clone(),
            };

            LanguageSample {
                language_id,
                file_extension: format!(".{}", ext),
                content,
                hover_line: 0,
                hover_char: 0,
                signature_line: 0,
                signature_char: 0,
                completion_line: 0,
                completion_char: 0,
                definition_line: 0,
                definition_char: 0,
                references_line: 0,
                references_char: 0,
                rename_line: 0,
                rename_char: 0,
            }
        } else if let Some(lang) = language {
            crate::languages::get_sample(&lang).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "Unknown language: {}. Supported: rust, c, cpp, python, d, zig, csharp, nim, hare, scheme, java, kotlin, crystal",
                    lang
                ),
            )
        })?
        } else {
            crate::languages::get_sample("rust").ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::Other, "Failed to get default sample")
            })?
        };

        Ok(HealthChecker {
            client,
            capabilities: ServerCapabilities::default(),
            results: Vec::new(),
            log_writer,
            server_name,
            sample,
        })
    }
    pub fn deinit(&mut self) {
        if let Some(ref mut writer) = self.log_writer {
            let _ = writer.flush();
        }
        self.client.deinit();
    }

    pub fn run_all_checks(&mut self) -> std::io::Result<Vec<CheckResult>> {
        self.check_initialize()?;
        self.check_did_open()?;
        self.check_publish_diagnostics()?;
        self.check_hover()?;
        self.check_signature_help()?;
        self.check_completion()?;
        self.check_definition()?;
        self.check_type_definition()?;
        self.check_implementation()?;
        self.check_references()?;
        self.check_document_symbol()?;
        self.check_workspace_symbol()?;
        self.check_formatting()?;
        self.check_code_action()?;
        self.check_rename()?;
        self.check_prepare_rename()?;
        self.check_inlay_hint()?;
        self.check_code_lens()?;
        self.check_semantic_tokens()?;
        self.check_folding_range()?;
        self.check_linked_editing_range()?;
        self.check_selection_range()?;
        self.check_document_highlight()?;
        self.check_did_change_configuration()?;
        self.check_did_change_workspace_folders()?;
        self.check_execute_command()?;
        self.check_shutdown()?;
        Ok(self.results.clone())
    }

    fn record(
        &mut self,
        name: &'static str,
        method: &'static str,
        status: CheckStatus,
        detail: &str,
        duration_ms: i64,
    ) {
        self.results.push(CheckResult {
            name,
            method,
            status,
            detail: detail.to_string(),
            duration_ms,
        });
    }

    fn record_with_error(
        &mut self,
        name: &'static str,
        method: &'static str,
        status: CheckStatus,
        display_detail: &str,
        actual_error: &str,
        duration_ms: i64,
    ) {
        if let Some(ref mut writer) = self.log_writer {
            let line = format!("{}:{} -> {}\n", self.server_name, name, actual_error);
            let _ = writer.write_all(line.as_bytes());
        }

        self.results.push(CheckResult {
            name,
            method,
            status,
            detail: display_detail.to_string(),
            duration_ms,
        });
    }

    fn check_initialize(&mut self) -> std::io::Result<()> {
        let t0 = Instant::now();

        let params = self.build_initialize_params();
        let id = self.client.send_request("initialize", Some(params))?;
        let resp = match self
            .client
            .read_response(id, Duration::from_millis(TIMEOUT_MS))?
        {
            Some(r) => {
                r
            }
            None => {
                self.record(
                    "Initialize",
                    "initialize",
                    CheckStatus::Timeout,
                    "no response",
                    t0.elapsed().as_millis() as i64,
                );
                return Ok(());
            }
        };

        let dt = t0.elapsed().as_millis() as i64;

        if let Some(error) = resp.get("error") {
            if !error.is_null() {
                let error_msg = extract_error_message(&resp);
                self.record_with_error(
                    "Initialize",
                    "initialize",
                    CheckStatus::Failed,
                    "server returned error",
                    &error_msg,
                    dt,
                );
                return Ok(());
            }
        }

        if let Some(result) = resp.get("result") {
            self.capabilities = ServerCapabilities::from_value(result);
        }

        let initialized_params = json!({});
        self.client
            .send_notification("initialized", Some(initialized_params))?;

        self.record(
            "Initialize",
            "initialize",
            CheckStatus::Passed,
            "handshake complete",
            dt,
        );
        Ok(())
    }

    fn check_did_open(&mut self) -> std::io::Result<()> {
        let t0 = Instant::now();

        let params = json!({
            "textDocument": {
                "uri": self.sample.uri(),
                "languageId": self.sample.language_id,
                "version": 1,
                "text": self.sample.content
            }
        });

        self.client
            .send_notification("textDocument/didOpen", Some(params))?;

        std::thread::sleep(Duration::from_millis(200));

        let dt = t0.elapsed().as_millis() as i64;
        self.record(
            "Open Document",
            "textDocument/didOpen",
            CheckStatus::Passed,
            "notification sent",
            dt,
        );
        Ok(())
    }

    fn check_hover(&mut self) -> std::io::Result<()> {
        if !self.capabilities.hover_provider {
            self.record(
                "Hover",
                "textDocument/hover",
                CheckStatus::Skipped,
                "not advertised",
                0,
            );
            return Ok(());
        }

        let t0 = Instant::now();
        let params = self.text_document_position(self.sample.hover_line, self.sample.hover_char);
        let id = self
            .client
            .send_request("textDocument/hover", Some(params))?;

        let resp = match self
            .client
            .read_response(id, Duration::from_millis(TIMEOUT_MS))?
        {
            Some(r) => r,
            None => {
                self.record(
                    "Hover",
                    "textDocument/hover",
                    CheckStatus::Timeout,
                    "no response",
                    t0.elapsed().as_millis() as i64,
                );
                return Ok(());
            }
        };

        let dt = t0.elapsed().as_millis() as i64;
        if resp.get("error").map(|e| !e.is_null()).unwrap_or(false) {
            let error_msg = extract_error_message(&resp);
            self.record_with_error(
                "Hover",
                "textDocument/hover",
                CheckStatus::Failed,
                "server error",
                &error_msg,
                dt,
            );
        } else {
            self.record(
                "Hover",
                "textDocument/hover",
                CheckStatus::Passed,
                "response received",
                dt,
            );
        }
        Ok(())
    }

    fn check_signature_help(&mut self) -> std::io::Result<()> {
        if !self.capabilities.signature_help_provider {
            self.record(
                "Signature Help",
                "textDocument/signatureHelp",
                CheckStatus::Skipped,
                "not advertised",
                0,
            );
            return Ok(());
        }

        let t0 = Instant::now();
        let params =
            self.text_document_position(self.sample.signature_line, self.sample.signature_char);
        let id = self
            .client
            .send_request("textDocument/signatureHelp", Some(params))?;

        let resp = match self
            .client
            .read_response(id, Duration::from_millis(TIMEOUT_MS))?
        {
            Some(r) => r,
            None => {
                self.record(
                    "Signature Help",
                    "textDocument/signatureHelp",
                    CheckStatus::Timeout,
                    "no response",
                    t0.elapsed().as_millis() as i64,
                );
                return Ok(());
            }
        };

        let dt = t0.elapsed().as_millis() as i64;
        if resp.get("error").map(|e| !e.is_null()).unwrap_or(false) {
            let error_msg = extract_error_message(&resp);
            self.record_with_error(
                "Signature Help",
                "textDocument/signatureHelp",
                CheckStatus::Failed,
                "server error",
                &error_msg,
                dt,
            );
        } else {
            self.record(
                "Signature Help",
                "textDocument/signatureHelp",
                CheckStatus::Passed,
                "response received",
                dt,
            );
        }
        Ok(())
    }

    fn check_completion(&mut self) -> std::io::Result<()> {
        if !self.capabilities.completion_provider {
            self.record(
                "Completion",
                "textDocument/completion",
                CheckStatus::Skipped,
                "not advertised",
                0,
            );
            return Ok(());
        }

        let t0 = Instant::now();
        let params =
            self.text_document_position(self.sample.completion_line, self.sample.completion_char);
        let id = self
            .client
            .send_request("textDocument/completion", Some(params))?;

        let resp = match self
            .client
            .read_response(id, Duration::from_millis(TIMEOUT_MS))?
        {
            Some(r) => r,
            None => {
                self.record(
                    "Completion",
                    "textDocument/completion",
                    CheckStatus::Timeout,
                    "no response",
                    t0.elapsed().as_millis() as i64,
                );
                return Ok(());
            }
        };

        let dt = t0.elapsed().as_millis() as i64;
        if resp.get("error").map(|e| !e.is_null()).unwrap_or(false) {
            let error_msg = extract_error_message(&resp);
            self.record_with_error(
                "Completion",
                "textDocument/completion",
                CheckStatus::Failed,
                "server error",
                &error_msg,
                dt,
            );
        } else {
            self.record(
                "Completion",
                "textDocument/completion",
                CheckStatus::Passed,
                "response received",
                dt,
            );
        }
        Ok(())
    }

    fn check_definition(&mut self) -> std::io::Result<()> {
        if !self.capabilities.definition_provider {
            self.record(
                "Go to Definition",
                "textDocument/definition",
                CheckStatus::Skipped,
                "not advertised",
                0,
            );
            return Ok(());
        }

        let t0 = Instant::now();
        let params =
            self.text_document_position(self.sample.definition_line, self.sample.definition_char);
        let id = self
            .client
            .send_request("textDocument/definition", Some(params))?;

        let resp = match self
            .client
            .read_response(id, Duration::from_millis(TIMEOUT_MS))?
        {
            Some(r) => r,
            None => {
                self.record(
                    "Go to Definition",
                    "textDocument/definition",
                    CheckStatus::Timeout,
                    "no response",
                    t0.elapsed().as_millis() as i64,
                );
                return Ok(());
            }
        };

        let dt = t0.elapsed().as_millis() as i64;
        if resp.get("error").map(|e| !e.is_null()).unwrap_or(false) {
            let error_msg = extract_error_message(&resp);
            self.record_with_error(
                "Go to Definition",
                "textDocument/definition",
                CheckStatus::Failed,
                "server error",
                &error_msg,
                dt,
            );
        } else {
            self.record(
                "Go to Definition",
                "textDocument/definition",
                CheckStatus::Passed,
                "response received",
                dt,
            );
        }
        Ok(())
    }

    fn check_references(&mut self) -> std::io::Result<()> {
        if !self.capabilities.references_provider {
            self.record(
                "Find References",
                "textDocument/references",
                CheckStatus::Skipped,
                "not advertised",
                0,
            );
            return Ok(());
        }

        let t0 = Instant::now();

        let params = json!({
            "textDocument": {
                "uri": self.sample.uri()
            },
            "position": {
                "line": self.sample.references_line,
                "character": self.sample.references_char
            },
            "context": {
                "includeDeclaration": true
            }
        });

        let id = self
            .client
            .send_request("textDocument/references", Some(params))?;

        let resp = match self
            .client
            .read_response(id, Duration::from_millis(TIMEOUT_MS))?
        {
            Some(r) => r,
            None => {
                self.record(
                    "Find References",
                    "textDocument/references",
                    CheckStatus::Timeout,
                    "no response",
                    t0.elapsed().as_millis() as i64,
                );
                return Ok(());
            }
        };

        let dt = t0.elapsed().as_millis() as i64;
        if resp.get("error").map(|e| !e.is_null()).unwrap_or(false) {
            let error_msg = extract_error_message(&resp);
            self.record_with_error(
                "Find References",
                "textDocument/references",
                CheckStatus::Failed,
                "server error",
                &error_msg,
                dt,
            );
        } else {
            self.record(
                "Find References",
                "textDocument/references",
                CheckStatus::Passed,
                "response received",
                dt,
            );
        }
        Ok(())
    }

    fn check_document_symbol(&mut self) -> std::io::Result<()> {
        if !self.capabilities.document_symbol_provider {
            self.record(
                "Document Symbols",
                "textDocument/documentSymbol",
                CheckStatus::Skipped,
                "not advertised",
                0,
            );
            return Ok(());
        }

        let t0 = Instant::now();
        let params = json!({
            "textDocument": {
                "uri": self.sample.uri()
            }
        });
        let id = self
            .client
            .send_request("textDocument/documentSymbol", Some(params))?;

        let resp = match self
            .client
            .read_response(id, Duration::from_millis(TIMEOUT_MS))?
        {
            Some(r) => r,
            None => {
                self.record(
                    "Document Symbols",
                    "textDocument/documentSymbol",
                    CheckStatus::Timeout,
                    "no response",
                    t0.elapsed().as_millis() as i64,
                );
                return Ok(());
            }
        };

        let dt = t0.elapsed().as_millis() as i64;
        if resp.get("error").map(|e| !e.is_null()).unwrap_or(false) {
            let error_msg = extract_error_message(&resp);
            self.record_with_error(
                "Document Symbols",
                "textDocument/documentSymbol",
                CheckStatus::Failed,
                "server error",
                &error_msg,
                dt,
            );
        } else {
            self.record(
                "Document Symbols",
                "textDocument/documentSymbol",
                CheckStatus::Passed,
                "response received",
                dt,
            );
        }
        Ok(())
    }

    fn check_formatting(&mut self) -> std::io::Result<()> {
        if !self.capabilities.document_formatting_provider {
            self.record(
                "Formatting",
                "textDocument/formatting",
                CheckStatus::Skipped,
                "not advertised",
                0,
            );
            return Ok(());
        }

        let t0 = Instant::now();
        let params = json!({
            "textDocument": {
                "uri": self.sample.uri()
            },
            "options": {
                "tabSize": 4,
                "insertSpaces": true
            }
        });
        let id = self
            .client
            .send_request("textDocument/formatting", Some(params))?;

        let resp = match self
            .client
            .read_response(id, Duration::from_millis(TIMEOUT_MS))?
        {
            Some(r) => r,
            None => {
                self.record(
                    "Formatting",
                    "textDocument/formatting",
                    CheckStatus::Timeout,
                    "no response",
                    t0.elapsed().as_millis() as i64,
                );
                return Ok(());
            }
        };

        let dt = t0.elapsed().as_millis() as i64;
        if resp.get("error").map(|e| !e.is_null()).unwrap_or(false) {
            let error_msg = extract_error_message(&resp);
            self.record_with_error(
                "Formatting",
                "textDocument/formatting",
                CheckStatus::Failed,
                "server error",
                &error_msg,
                dt,
            );
        } else {
            self.record(
                "Formatting",
                "textDocument/formatting",
                CheckStatus::Passed,
                "response received",
                dt,
            );
        }
        Ok(())
    }

    fn check_code_action(&mut self) -> std::io::Result<()> {
        if !self.capabilities.code_action_provider {
            self.record(
                "Code Actions",
                "textDocument/codeAction",
                CheckStatus::Skipped,
                "not advertised",
                0,
            );
            return Ok(());
        }

        let t0 = Instant::now();
        let params = json!({
            "textDocument": {
                "uri": self.sample.uri()
            },
            "range": {
                "start": {
                    "line": 0,
                    "character": 0
                },
                "end": {
                    "line": 0,
                    "character": 10
                }
            },
            "context": {
                "diagnostics": []
            }
        });
        let id = self
            .client
            .send_request("textDocument/codeAction", Some(params))?;

        let resp = match self
            .client
            .read_response(id, Duration::from_millis(TIMEOUT_MS))?
        {
            Some(r) => r,
            None => {
                self.record(
                    "Code Actions",
                    "textDocument/codeAction",
                    CheckStatus::Timeout,
                    "no response",
                    t0.elapsed().as_millis() as i64,
                );
                return Ok(());
            }
        };

        let dt = t0.elapsed().as_millis() as i64;
        if resp.get("error").map(|e| !e.is_null()).unwrap_or(false) {
            let error_msg = extract_error_message(&resp);
            self.record_with_error(
                "Code Actions",
                "textDocument/codeAction",
                CheckStatus::Failed,
                "server error",
                &error_msg,
                dt,
            );
        } else {
            self.record(
                "Code Actions",
                "textDocument/codeAction",
                CheckStatus::Passed,
                "response received",
                dt,
            );
        }
        Ok(())
    }

    fn check_rename(&mut self) -> std::io::Result<()> {
        if !self.capabilities.rename_provider {
            self.record(
                "Rename Symbol",
                "textDocument/rename",
                CheckStatus::Skipped,
                "not advertised",
                0,
            );
            return Ok(());
        }

        let t0 = Instant::now();
        let params = json!({
            "textDocument": {
                "uri": self.sample.uri()
            },
            "position": {
                "line": self.sample.rename_line,
                "character": self.sample.rename_char
            },
            "newName": "sum"
        });
        let id = self
            .client
            .send_request("textDocument/rename", Some(params))?;

        let resp = match self
            .client
            .read_response(id, Duration::from_millis(TIMEOUT_MS))?
        {
            Some(r) => r,
            None => {
                self.record(
                    "Rename Symbol",
                    "textDocument/rename",
                    CheckStatus::Timeout,
                    "no response",
                    t0.elapsed().as_millis() as i64,
                );
                return Ok(());
            }
        };

        let dt = t0.elapsed().as_millis() as i64;
        if resp.get("error").map(|e| !e.is_null()).unwrap_or(false) {
            let error_msg = extract_error_message(&resp);
            self.record_with_error(
                "Rename Symbol",
                "textDocument/rename",
                CheckStatus::Failed,
                "server error",
                &error_msg,
                dt,
            );
        } else {
            self.record(
                "Rename Symbol",
                "textDocument/rename",
                CheckStatus::Passed,
                "response received",
                dt,
            );
        }
        Ok(())
    }

    fn check_inlay_hint(&mut self) -> std::io::Result<()> {
        if !self.capabilities.inlay_hint_provider {
            self.record(
                "Inlay Hints",
                "textDocument/inlayHint",
                CheckStatus::Skipped,
                "not advertised",
                0,
            );
            return Ok(());
        }

        let t0 = Instant::now();
        let params = json!({
            "textDocument": {
                "uri": self.sample.uri()
            },
            "range": {
                "start": {
                    "line": 0,
                    "character": 0
                },
                "end": {
                    "line": 10,
                    "character": 0
                }
            }
        });
        let id = self
            .client
            .send_request("textDocument/inlayHint", Some(params))?;

        let resp = match self
            .client
            .read_response(id, Duration::from_millis(TIMEOUT_MS))?
        {
            Some(r) => r,
            None => {
                self.record(
                    "Inlay Hints",
                    "textDocument/inlayHint",
                    CheckStatus::Timeout,
                    "no response",
                    t0.elapsed().as_millis() as i64,
                );
                return Ok(());
            }
        };

        let dt = t0.elapsed().as_millis() as i64;
        if resp.get("error").map(|e| !e.is_null()).unwrap_or(false) {
            let error_msg = extract_error_message(&resp);
            self.record_with_error(
                "Inlay Hints",
                "textDocument/inlayHint",
                CheckStatus::Failed,
                "server error",
                &error_msg,
                dt,
            );
        } else {
            self.record(
                "Inlay Hints",
                "textDocument/inlayHint",
                CheckStatus::Passed,
                "response received",
                dt,
            );
        }
        Ok(())
    }

    fn check_workspace_symbol(&mut self) -> std::io::Result<()> {
        if !self.capabilities.workspace_symbol_provider {
            self.record(
                "Workspace Symbols",
                "workspace/symbol",
                CheckStatus::Skipped,
                "not advertised",
                0,
            );
            return Ok(());
        }

        let t0 = Instant::now();
        let params = json!({
            "query": "add"
        });
        let id = self.client.send_request("workspace/symbol", Some(params))?;

        let resp = match self
            .client
            .read_response(id, Duration::from_millis(TIMEOUT_MS))?
        {
            Some(r) => r,
            None => {
                self.record(
                    "Workspace Symbols",
                    "workspace/symbol",
                    CheckStatus::Timeout,
                    "no response",
                    t0.elapsed().as_millis() as i64,
                );
                return Ok(());
            }
        };

        let dt = t0.elapsed().as_millis() as i64;
        if resp.get("error").map(|e| !e.is_null()).unwrap_or(false) {
            let error_msg = extract_error_message(&resp);
            self.record_with_error(
                "Workspace Symbols",
                "workspace/symbol",
                CheckStatus::Failed,
                "server error",
                &error_msg,
                dt,
            );
        } else {
            self.record(
                "Workspace Symbols",
                "workspace/symbol",
                CheckStatus::Passed,
                "response received",
                dt,
            );
        }
        Ok(())
    }

    fn check_did_change_configuration(&mut self) -> std::io::Result<()> {
        let t0 = Instant::now();

        let params = json!({
            "settings": {
                "lsp": {
                    "enabled": true
                }
            }
        });

        self.client
            .send_notification("workspace/didChangeConfiguration", Some(params))?;

        std::thread::sleep(Duration::from_millis(100));

        let dt = t0.elapsed().as_millis() as i64;
        self.record(
            "DidChangeConfiguration",
            "workspace/didChangeConfiguration",
            CheckStatus::Passed,
            "notification sent",
            dt,
        );
        Ok(())
    }

    fn check_did_change_workspace_folders(&mut self) -> std::io::Result<()> {
        let t0 = Instant::now();

        let params = json!({
            "event": {
                "added": [
                    {
                        "uri": "file:///tmp/workspace",
                        "name": "workspace"
                    }
                ],
                "removed": []
            }
        });

        self.client
            .send_notification("workspace/didChangeWorkspaceFolders", Some(params))?;

        std::thread::sleep(Duration::from_millis(100));

        let dt = t0.elapsed().as_millis() as i64;
        self.record(
            "DidChangeWorkspaceFolders",
            "workspace/didChangeWorkspaceFolders",
            CheckStatus::Passed,
            "notification sent",
            dt,
        );
        Ok(())
    }

    fn check_execute_command(&mut self) -> std::io::Result<()> {
        if !self.capabilities.execute_command_provider {
            self.record(
                "Execute Command",
                "workspace/executeCommand",
                CheckStatus::Skipped,
                "not advertised",
                0,
            );
            return Ok(());
        }

        let t0 = Instant::now();
        let params = json!({
            "command": "_clangd.testCommand",
            "arguments": []
        });
        let id = self
            .client
            .send_request("workspace/executeCommand", Some(params))?;

        let resp = match self
            .client
            .read_response(id, Duration::from_millis(TIMEOUT_MS))?
        {
            Some(r) => r,
            None => {
                self.record(
                    "Execute Command",
                    "workspace/executeCommand",
                    CheckStatus::Timeout,
                    "no response",
                    t0.elapsed().as_millis() as i64,
                );
                return Ok(());
            }
        };

        let dt = t0.elapsed().as_millis() as i64;
        if resp.get("error").map(|e| !e.is_null()).unwrap_or(false) {
            let error_obj = resp.get("error").unwrap();
            let error_code = error_obj.get("code").and_then(|c| c.as_i64());
            if let Some(code) = error_code {
                if code == -32601 || code == -32602 {
                    self.record(
                        "Execute Command",
                        "workspace/executeCommand",
                        CheckStatus::Passed,
                        "protocol functional (command not supported)",
                        dt,
                    );
                    return Ok(());
                }
            }
            let error_msg = extract_error_message(&resp);
            self.record_with_error(
                "Execute Command",
                "workspace/executeCommand",
                CheckStatus::Failed,
                "server error",
                &error_msg,
                dt,
            );
        } else {
            self.record(
                "Execute Command",
                "workspace/executeCommand",
                CheckStatus::Passed,
                "response received",
                dt,
            );
        }
        Ok(())
    }

    fn check_type_definition(&mut self) -> std::io::Result<()> {
        if !self.capabilities.type_definition_provider {
            self.record(
                "Go to Type Definition",
                "textDocument/typeDefinition",
                CheckStatus::Skipped,
                "not advertised",
                0,
            );
            return Ok(());
        }

        let t0 = Instant::now();
        let params =
            self.text_document_position(self.sample.definition_line, self.sample.definition_char);
        let id = self
            .client
            .send_request("textDocument/typeDefinition", Some(params))?;

        let resp = match self
            .client
            .read_response(id, Duration::from_millis(TIMEOUT_MS))?
        {
            Some(r) => r,
            None => {
                self.record(
                    "Go to Type Definition",
                    "textDocument/typeDefinition",
                    CheckStatus::Timeout,
                    "no response",
                    t0.elapsed().as_millis() as i64,
                );
                return Ok(());
            }
        };

        let dt = t0.elapsed().as_millis() as i64;
        if resp.get("error").map(|e| !e.is_null()).unwrap_or(false) {
            let error_msg = extract_error_message(&resp);
            self.record_with_error(
                "Go to Type Definition",
                "textDocument/typeDefinition",
                CheckStatus::Failed,
                "server error",
                &error_msg,
                dt,
            );
        } else {
            self.record(
                "Go to Type Definition",
                "textDocument/typeDefinition",
                CheckStatus::Passed,
                "response received",
                dt,
            );
        }
        Ok(())
    }

    fn check_implementation(&mut self) -> std::io::Result<()> {
        if !self.capabilities.implementation_provider {
            self.record(
                "Go to Implementation",
                "textDocument/implementation",
                CheckStatus::Skipped,
                "not advertised",
                0,
            );
            return Ok(());
        }

        let t0 = Instant::now();
        let params =
            self.text_document_position(self.sample.definition_line, self.sample.definition_char);
        let id = self
            .client
            .send_request("textDocument/implementation", Some(params))?;

        let resp = match self
            .client
            .read_response(id, Duration::from_millis(TIMEOUT_MS))?
        {
            Some(r) => r,
            None => {
                self.record(
                    "Go to Implementation",
                    "textDocument/implementation",
                    CheckStatus::Timeout,
                    "no response",
                    t0.elapsed().as_millis() as i64,
                );
                return Ok(());
            }
        };

        let dt = t0.elapsed().as_millis() as i64;
        if resp.get("error").map(|e| !e.is_null()).unwrap_or(false) {
            let error_msg = extract_error_message(&resp);
            self.record_with_error(
                "Go to Implementation",
                "textDocument/implementation",
                CheckStatus::Failed,
                "server error",
                &error_msg,
                dt,
            );
        } else {
            self.record(
                "Go to Implementation",
                "textDocument/implementation",
                CheckStatus::Passed,
                "response received",
                dt,
            );
        }
        Ok(())
    }

    fn check_document_highlight(&mut self) -> std::io::Result<()> {
        if !self.capabilities.document_highlight_provider {
            self.record(
                "Document Highlight",
                "textDocument/documentHighlight",
                CheckStatus::Skipped,
                "not advertised",
                0,
            );
            return Ok(());
        }

        let t0 = Instant::now();
        let params =
            self.text_document_position(self.sample.references_line, self.sample.references_char);
        let id = self
            .client
            .send_request("textDocument/documentHighlight", Some(params))?;

        let resp = match self
            .client
            .read_response(id, Duration::from_millis(TIMEOUT_MS))?
        {
            Some(r) => r,
            None => {
                self.record(
                    "Document Highlight",
                    "textDocument/documentHighlight",
                    CheckStatus::Timeout,
                    "no response",
                    t0.elapsed().as_millis() as i64,
                );
                return Ok(());
            }
        };

        let dt = t0.elapsed().as_millis() as i64;
        if resp.get("error").map(|e| !e.is_null()).unwrap_or(false) {
            let error_msg = extract_error_message(&resp);
            self.record_with_error(
                "Document Highlight",
                "textDocument/documentHighlight",
                CheckStatus::Failed,
                "server error",
                &error_msg,
                dt,
            );
        } else {
            self.record(
                "Document Highlight",
                "textDocument/documentHighlight",
                CheckStatus::Passed,
                "response received",
                dt,
            );
        }
        Ok(())
    }

    fn check_selection_range(&mut self) -> std::io::Result<()> {
        if !self.capabilities.selection_range_provider {
            self.record(
                "Selection Range",
                "textDocument/selectionRange",
                CheckStatus::Skipped,
                "not advertised",
                0,
            );
            return Ok(());
        }

        let t0 = Instant::now();
        let params = json!({
            "textDocument": {
                "uri": self.sample.uri()
            },
            "positions": [
                {
                    "line": self.sample.hover_line,
                    "character": self.sample.hover_char
                }
            ]
        });
        let id = self
            .client
            .send_request("textDocument/selectionRange", Some(params))?;

        let resp = match self
            .client
            .read_response(id, Duration::from_millis(TIMEOUT_MS))?
        {
            Some(r) => r,
            None => {
                self.record(
                    "Selection Range",
                    "textDocument/selectionRange",
                    CheckStatus::Timeout,
                    "no response",
                    t0.elapsed().as_millis() as i64,
                );
                return Ok(());
            }
        };

        let dt = t0.elapsed().as_millis() as i64;
        if resp.get("error").map(|e| !e.is_null()).unwrap_or(false) {
            let error_msg = extract_error_message(&resp);
            self.record_with_error(
                "Selection Range",
                "textDocument/selectionRange",
                CheckStatus::Failed,
                "server error",
                &error_msg,
                dt,
            );
        } else {
            self.record(
                "Selection Range",
                "textDocument/selectionRange",
                CheckStatus::Passed,
                "response received",
                dt,
            );
        }
        Ok(())
    }

    fn check_folding_range(&mut self) -> std::io::Result<()> {
        if !self.capabilities.folding_range_provider {
            self.record(
                "Folding Range",
                "textDocument/foldingRange",
                CheckStatus::Skipped,
                "not advertised",
                0,
            );
            return Ok(());
        }

        let t0 = Instant::now();
        let params = json!({
            "textDocument": {
                "uri": self.sample.uri()
            }
        });
        let id = self
            .client
            .send_request("textDocument/foldingRange", Some(params))?;

        let resp = match self
            .client
            .read_response(id, Duration::from_millis(TIMEOUT_MS))?
        {
            Some(r) => r,
            None => {
                self.record(
                    "Folding Range",
                    "textDocument/foldingRange",
                    CheckStatus::Timeout,
                    "no response",
                    t0.elapsed().as_millis() as i64,
                );
                return Ok(());
            }
        };

        let dt = t0.elapsed().as_millis() as i64;
        if resp.get("error").map(|e| !e.is_null()).unwrap_or(false) {
            let error_msg = extract_error_message(&resp);
            self.record_with_error(
                "Folding Range",
                "textDocument/foldingRange",
                CheckStatus::Failed,
                "server error",
                &error_msg,
                dt,
            );
        } else {
            self.record(
                "Folding Range",
                "textDocument/foldingRange",
                CheckStatus::Passed,
                "response received",
                dt,
            );
        }
        Ok(())
    }

    fn check_linked_editing_range(&mut self) -> std::io::Result<()> {
        if !self.capabilities.linked_editing_range_provider {
            self.record(
                "Linked Editing Range",
                "textDocument/linkedEditingRange",
                CheckStatus::Skipped,
                "not advertised",
                0,
            );
            return Ok(());
        }

        let t0 = Instant::now();
        let params = json!({
            "textDocument": {
                "uri": self.sample.uri()
            },
            "position": {
                "line": self.sample.hover_line,
                "character": self.sample.hover_char
            }
        });
        let id = self
            .client
            .send_request("textDocument/linkedEditingRange", Some(params))?;

        let resp = match self
            .client
            .read_response(id, Duration::from_millis(TIMEOUT_MS))?
        {
            Some(r) => r,
            None => {
                self.record(
                    "Linked Editing Range",
                    "textDocument/linkedEditingRange",
                    CheckStatus::Timeout,
                    "no response",
                    t0.elapsed().as_millis() as i64,
                );
                return Ok(());
            }
        };

        let dt = t0.elapsed().as_millis() as i64;
        if resp.get("error").map(|e| !e.is_null()).unwrap_or(false) {
            let error_msg = extract_error_message(&resp);
            self.record_with_error(
                "Linked Editing Range",
                "textDocument/linkedEditingRange",
                CheckStatus::Failed,
                "server error",
                &error_msg,
                dt,
            );
        } else {
            self.record(
                "Linked Editing Range",
                "textDocument/linkedEditingRange",
                CheckStatus::Passed,
                "response received",
                dt,
            );
        }
        Ok(())
    }

    fn check_semantic_tokens(&mut self) -> std::io::Result<()> {
        if !self.capabilities.semantic_tokens_provider {
            self.record(
                "Semantic Tokens",
                "textDocument/semanticTokens/full",
                CheckStatus::Skipped,
                "not advertised",
                0,
            );
            return Ok(());
        }

        let t0 = Instant::now();
        let params = json!({
            "textDocument": {
                "uri": self.sample.uri()
            }
        });
        let id = self
            .client
            .send_request("textDocument/semanticTokens/full", Some(params))?;

        let resp = match self
            .client
            .read_response(id, Duration::from_millis(TIMEOUT_MS))?
        {
            Some(r) => r,
            None => {
                self.record(
                    "Semantic Tokens",
                    "textDocument/semanticTokens/full",
                    CheckStatus::Timeout,
                    "no response",
                    t0.elapsed().as_millis() as i64,
                );
                return Ok(());
            }
        };

        let dt = t0.elapsed().as_millis() as i64;
        if resp.get("error").map(|e| !e.is_null()).unwrap_or(false) {
            let error_msg = extract_error_message(&resp);
            self.record_with_error(
                "Semantic Tokens",
                "textDocument/semanticTokens/full",
                CheckStatus::Failed,
                "server error",
                &error_msg,
                dt,
            );
        } else {
            self.record(
                "Semantic Tokens",
                "textDocument/semanticTokens/full",
                CheckStatus::Passed,
                "response received",
                dt,
            );
        }
        Ok(())
    }

    fn check_code_lens(&mut self) -> std::io::Result<()> {
        if !self.capabilities.code_lens_provider {
            self.record(
                "Code Lens",
                "textDocument/codeLens",
                CheckStatus::Skipped,
                "not advertised",
                0,
            );
            return Ok(());
        }

        let t0 = Instant::now();
        let params = json!({
            "textDocument": {
                "uri": self.sample.uri()
            }
        });
        let id = self
            .client
            .send_request("textDocument/codeLens", Some(params))?;

        let resp = match self
            .client
            .read_response(id, Duration::from_millis(TIMEOUT_MS))?
        {
            Some(r) => r,
            None => {
                self.record(
                    "Code Lens",
                    "textDocument/codeLens",
                    CheckStatus::Timeout,
                    "no response",
                    t0.elapsed().as_millis() as i64,
                );
                return Ok(());
            }
        };

        let dt = t0.elapsed().as_millis() as i64;
        if resp.get("error").map(|e| !e.is_null()).unwrap_or(false) {
            let error_msg = extract_error_message(&resp);
            self.record_with_error(
                "Code Lens",
                "textDocument/codeLens",
                CheckStatus::Failed,
                "server error",
                &error_msg,
                dt,
            );
        } else {
            self.record(
                "Code Lens",
                "textDocument/codeLens",
                CheckStatus::Passed,
                "response received",
                dt,
            );
        }
        Ok(())
    }

    fn check_prepare_rename(&mut self) -> std::io::Result<()> {
        if !self.capabilities.prepare_rename_provider {
            self.record(
                "Prepare Rename",
                "textDocument/prepareRename",
                CheckStatus::Skipped,
                "not advertised",
                0,
            );
            return Ok(());
        }

        let t0 = Instant::now();
        let params = self.text_document_position(self.sample.rename_line, self.sample.rename_char);
        let id = self
            .client
            .send_request("textDocument/prepareRename", Some(params))?;

        let resp = match self
            .client
            .read_response(id, Duration::from_millis(TIMEOUT_MS))?
        {
            Some(r) => r,
            None => {
                self.record(
                    "Prepare Rename",
                    "textDocument/prepareRename",
                    CheckStatus::Timeout,
                    "no response",
                    t0.elapsed().as_millis() as i64,
                );
                return Ok(());
            }
        };

        let dt = t0.elapsed().as_millis() as i64;
        if resp.get("error").map(|e| !e.is_null()).unwrap_or(false) {
            let error_msg = extract_error_message(&resp);
            self.record_with_error(
                "Prepare Rename",
                "textDocument/prepareRename",
                CheckStatus::Failed,
                "server error",
                &error_msg,
                dt,
            );
        } else {
            self.record(
                "Prepare Rename",
                "textDocument/prepareRename",
                CheckStatus::Passed,
                "response received",
                dt,
            );
        }
        Ok(())
    }

    fn check_publish_diagnostics(&mut self) -> std::io::Result<()> {
        let t0 = Instant::now();

        std::thread::sleep(Duration::from_millis(500));

        let _ = self.client.read_message(Duration::from_millis(100));
        let dt = t0.elapsed().as_millis() as i64;

        if !self.capabilities.publish_diagnostics_provider {
            self.record(
                "Publish Diagnostics",
                "textDocument/publishDiagnostics",
                CheckStatus::Skipped,
                "not advertised",
                dt,
            );
        } else {
            self.record(
                "Publish Diagnostics",
                "textDocument/publishDiagnostics",
                CheckStatus::Passed,
                "provider registered",
                dt,
            );
        }
        Ok(())
    }

    fn check_shutdown(&mut self) -> std::io::Result<()> {
        let t0 = Instant::now();

        let id = self.client.send_request("shutdown", None)?;
        let resp = match self
            .client
            .read_response(id, Duration::from_millis(TIMEOUT_MS))?
        {
            Some(r) => r,
            None => {
                self.record(
                    "Shutdown",
                    "shutdown/exit",
                    CheckStatus::Timeout,
                    "no response",
                    t0.elapsed().as_millis() as i64,
                );
                return Ok(());
            }
        };

        self.client.send_notification("exit", None)?;

        let dt = t0.elapsed().as_millis() as i64;
        if resp.get("error").map(|e| !e.is_null()).unwrap_or(false) {
            let error_msg = extract_error_message(&resp);
            self.record_with_error(
                "Shutdown",
                "shutdown/exit",
                CheckStatus::Failed,
                "server error",
                &error_msg,
                dt,
            );
        } else {
            self.record(
                "Shutdown",
                "shutdown/exit",
                CheckStatus::Passed,
                "clean exit",
                dt,
            );
        }
        Ok(())
    }

    fn build_initialize_params(&self) -> Value {
        json!({
            "processId": std::process::id() as i64,
            "rootUri": null,
            "capabilities": {
                "textDocument": {},
                "workspace": {}
            }
        })
    }

    fn text_document_position(&self, line: i64, character: i64) -> Value {
        json!({
            "textDocument": {
                "uri": self.sample.uri()
            },
            "position": {
                "line": line,
                "character": character
            }
        })
    }
}
