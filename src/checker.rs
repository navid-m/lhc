use crate::lsp::{Client, ServerCapabilities};
use serde_json::{json, Value};
use std::time::{Duration, Instant};

pub const TIMEOUT_MS: u64 = 5000;
const DOC_URI: &str = "file:///tmp/lsp_health_check.rs";
const DOC_CONTENT: &str = r#"fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {
    let x = add(1, 2);
    let _ = x;
}
"#;

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
}

impl HealthChecker {
    pub fn init(server_path: &str, server_args: &[String]) -> std::io::Result<Self> {
        let client = Client::init(server_path, server_args)?;
        Ok(HealthChecker {
            client,
            capabilities: ServerCapabilities::default(),
            results: Vec::new(),
        })
    }

    pub fn deinit(&mut self) {
        self.client.deinit();
    }

    pub fn run_all_checks(&mut self) -> std::io::Result<Vec<CheckResult>> {
        self.check_initialize()?;
        self.check_did_open()?;
        self.check_hover()?;
        self.check_signature_help()?;
        self.check_completion()?;
        self.check_definition()?;
        self.check_references()?;
        self.check_document_symbol()?;
        self.check_formatting()?;
        self.check_code_action()?;
        self.check_rename()?;
        self.check_inlay_hint()?;
        self.check_workspace_symbol()?;
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

    fn check_initialize(&mut self) -> std::io::Result<()> {
        let t0 = Instant::now();

        let params = self.build_initialize_params();
        let id = self.client.send_request("initialize", Some(params))?;

        let resp = match self
            .client
            .read_response(id, Duration::from_millis(TIMEOUT_MS))?
        {
            Some(r) => r,
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
                self.record(
                    "Initialize",
                    "initialize",
                    CheckStatus::Failed,
                    "server returned error",
                    dt,
                );
                return Ok(());
            }
        }

        if let Some(result) = resp.get("result") {
            self.capabilities = ServerCapabilities::from_value(result);
        }

        let initialized_params = json!({});
        self.client.send_notification("initialized", Some(initialized_params))?;

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
                "uri": DOC_URI,
                "languageId": "rust",
                "version": 1,
                "text": DOC_CONTENT
            }
        });

        self.client.send_notification("textDocument/didOpen", Some(params))?;

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
        let params = self.text_document_position(2, 7);
        let id = self.client.send_request("textDocument/hover", Some(params))?;

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
            self.record(
                "Hover",
                "textDocument/hover",
                CheckStatus::Failed,
                "server error",
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
        let params = self.text_document_position(8, 19); // inside add(
        let id = self.client.send_request("textDocument/signatureHelp", Some(params))?;

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
            self.record(
                "Signature Help",
                "textDocument/signatureHelp",
                CheckStatus::Failed,
                "server error",
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
        let params = self.text_document_position(8, 14); // after 'let x = '
        let id = self.client.send_request("textDocument/completion", Some(params))?;

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
            self.record(
                "Completion",
                "textDocument/completion",
                CheckStatus::Failed,
                "server error",
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
        let params = self.text_document_position(8, 18); // on 'add' call
        let id = self.client.send_request("textDocument/definition", Some(params))?;

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
            self.record(
                "Go to Definition",
                "textDocument/definition",
                CheckStatus::Failed,
                "server error",
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
                "uri": DOC_URI
            },
            "position": {
                "line": 2,
                "character": 7
            },
            "context": {
                "includeDeclaration": true
            }
        });

        let id = self.client.send_request("textDocument/references", Some(params))?;

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
            self.record(
                "Find References",
                "textDocument/references",
                CheckStatus::Failed,
                "server error",
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
                "uri": DOC_URI
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
            self.record(
                "Document Symbols",
                "textDocument/documentSymbol",
                CheckStatus::Failed,
                "server error",
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
                "uri": DOC_URI
            },
            "options": {
                "tabSize": 4,
                "insertSpaces": true
            }
        });
        let id = self.client.send_request("textDocument/formatting", Some(params))?;

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
            self.record(
                "Formatting",
                "textDocument/formatting",
                CheckStatus::Failed,
                "server error",
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
                "uri": DOC_URI
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
        let id = self.client.send_request("textDocument/codeAction", Some(params))?;

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
            self.record(
                "Code Actions",
                "textDocument/codeAction",
                CheckStatus::Failed,
                "server error",
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
                "uri": DOC_URI
            },
            "position": {
                "line": 2,
                "character": 7
            },
            "newName": "sum"
        });
        let id = self.client.send_request("textDocument/rename", Some(params))?;

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
            self.record(
                "Rename Symbol",
                "textDocument/rename",
                CheckStatus::Failed,
                "server error",
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
                "uri": DOC_URI
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
        let id = self.client.send_request("textDocument/inlayHint", Some(params))?;

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
            self.record(
                "Inlay Hints",
                "textDocument/inlayHint",
                CheckStatus::Failed,
                "server error",
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
            self.record(
                "Workspace Symbols",
                "workspace/symbol",
                CheckStatus::Failed,
                "server error",
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
            self.record(
                "Shutdown",
                "shutdown/exit",
                CheckStatus::Failed,
                "server error",
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
            "clientInfo": {
                "name": "lsp-health-checker",
                "version": "1.0.0"
            },
            "rootUri": "file:///tmp",
            "capabilities": {
                "textDocument": {
                    "hover": {
                        "contentFormat": ["markdown", "plaintext"]
                    },
                    "signatureHelp": {
                        "signatureInformation": {
                            "documentationFormat": ["markdown"]
                        }
                    },
                    "completion": {
                        "completionItem": {
                            "snippetSupport": true
                        }
                    },
                    "definition": {
                        "dynamicRegistration": false
                    },
                    "references": {
                        "dynamicRegistration": false
                    },
                    "documentSymbol": {
                        "hierarchicalDocumentSymbolSupport": true
                    },
                    "formatting": {
                        "dynamicRegistration": false
                    },
                    "codeAction": {
                        "dynamicRegistration": false
                    },
                    "rename": {
                        "dynamicRegistration": false
                    },
                    "inlayHint": {
                        "dynamicRegistration": false
                    }
                },
                "workspace": {
                    "symbol": {
                        "dynamicRegistration": false
                    }
                }
            },
            "trace": "off",
            "workspaceFolders": Value::Null
        })
    }

    fn text_document_position(&self, line: i64, character: i64) -> Value {
        json!({
            "textDocument": {
                "uri": DOC_URI
            },
            "position": {
                "line": line,
                "character": character
            }
        })
    }
}
