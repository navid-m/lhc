use memchr::memmem;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: i64,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Default)]
pub struct ServerCapabilities {
    pub hover_provider: bool,
    pub signature_help_provider: bool,
    pub completion_provider: bool,
    pub definition_provider: bool,
    pub type_definition_provider: bool,
    pub implementation_provider: bool,
    pub references_provider: bool,
    pub document_symbol_provider: bool,
    pub workspace_symbol_provider: bool,
    pub document_formatting_provider: bool,
    pub code_action_provider: bool,
    pub rename_provider: bool,
    pub prepare_rename_provider: bool,
    pub inlay_hint_provider: bool,
    pub code_lens_provider: bool,
    pub semantic_tokens_provider: bool,
    pub folding_range_provider: bool,
    pub linked_editing_range_provider: bool,
    pub selection_range_provider: bool,
    pub document_highlight_provider: bool,
    pub publish_diagnostics_provider: bool,
    pub execute_command_provider: bool,
    pub did_change_configuration_provider: bool,
    pub did_change_workspace_folders_provider: bool,
    pub text_document_sync_incremental: bool,
    pub text_document_sync_full: bool,
    pub will_save_provider: bool,
    pub will_save_wait_until_provider: bool,
    pub did_save_provider: bool,
    pub workspace_configuration_provider: bool,
    pub did_change_watched_files_provider: bool,
    pub completion_item_resolve_provider: bool,
    pub code_lens_resolve_provider: bool,
    pub document_link_provider: bool,
    pub document_link_resolve_provider: bool,
    pub color_provider: bool,
    pub declaration_provider: bool,
    pub type_hierarchy_provider: bool,
    pub call_hierarchy_provider: bool,
    pub semantic_tokens_range_provider: bool,
    pub inline_completion_provider: bool,
}

impl ServerCapabilities {
    pub fn from_value(value: &Value) -> Self {
        let Some(obj) = value.as_object() else {
            return ServerCapabilities::default();
        };

        let Some(capabilities) = obj.get("capabilities") else {
            return ServerCapabilities::default();
        };

        let Some(caps_obj) = capabilities.as_object() else {
            return ServerCapabilities::default();
        };

        let mut caps = ServerCapabilities::default();

        caps.hover_provider = is_truthy(caps_obj.get("hoverProvider"));
        caps.signature_help_provider = caps_obj.get("signatureHelpProvider").is_some();
        caps.completion_provider = caps_obj.get("completionProvider").is_some();
        caps.definition_provider = is_truthy(caps_obj.get("definitionProvider"));
        caps.type_definition_provider = is_truthy(caps_obj.get("typeDefinitionProvider"));
        caps.implementation_provider = is_truthy(caps_obj.get("implementationProvider"));
        caps.references_provider = is_truthy(caps_obj.get("referencesProvider"));
        caps.document_symbol_provider = is_truthy(caps_obj.get("documentSymbolProvider"));
        caps.document_formatting_provider = is_truthy(caps_obj.get("documentFormattingProvider"));
        caps.code_action_provider = is_truthy(caps_obj.get("codeActionProvider"));
        caps.rename_provider = is_truthy(caps_obj.get("renameProvider"));
        caps.prepare_rename_provider = is_truthy(caps_obj.get("prepareRenameProvider"));
        caps.inlay_hint_provider = is_truthy(caps_obj.get("inlayHintProvider"));
        caps.code_lens_provider = is_truthy(caps_obj.get("codeLensProvider"));
        caps.semantic_tokens_provider = caps_obj.get("semanticTokensProvider").is_some();
        caps.folding_range_provider = caps_obj.get("foldingRangeProvider").is_some();
        caps.linked_editing_range_provider = caps_obj.get("linkedEditingRangeProvider").is_some();
        caps.selection_range_provider = caps_obj.get("selectionRangeProvider").is_some();
        caps.document_highlight_provider = is_truthy(caps_obj.get("documentHighlightProvider"));
        caps.publish_diagnostics_provider = caps_obj.get("publishDiagnosticsProvider").is_some();
        caps.execute_command_provider = caps_obj.get("executeCommandProvider").is_some();
        caps.did_change_configuration_provider = caps_obj
            .get("workspace")
            .and_then(|w| w.as_object())
            .and_then(|w| w.get("workspaceFolders"))
            .map(|wf| !wf.is_null())
            .unwrap_or(false);
        caps.did_change_workspace_folders_provider = caps_obj
            .get("workspace")
            .and_then(|w| w.as_object())
            .and_then(|w| w.get("workspaceFolders"))
            .map(|wf| !wf.is_null())
            .unwrap_or(false);

        if let Some(ws) = caps_obj.get("workspace") {
            if let Some(ws_obj) = ws.as_object() {
                if let Some(sym) = ws_obj.get("symbol") {
                    caps.workspace_symbol_provider = !sym.is_null();
                }
                if let Some(cfg) = ws_obj.get("fileOperations") {
                    caps.workspace_configuration_provider = !cfg.is_null();
                }
            }
        }
        if let Some(sync) = caps_obj.get("textDocumentSync") {
            match sync {
                Value::Number(n) => {
                    let kind = n.as_u64().unwrap_or(0);
                    caps.text_document_sync_full = kind == 1 || kind == 2;
                    caps.text_document_sync_incremental = kind == 2;
                    caps.will_save_provider = kind > 0;
                    caps.will_save_wait_until_provider = kind > 0;
                    caps.did_save_provider = kind > 0;
                }
                Value::Object(sync_obj) => {
                    let change = sync_obj.get("change").and_then(|v| v.as_u64()).unwrap_or(0);
                    caps.text_document_sync_full = change == 1 || change == 2;
                    caps.text_document_sync_incremental = change == 2;
                    caps.will_save_provider = sync_obj
                        .get("willSave")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    caps.will_save_wait_until_provider = sync_obj
                        .get("willSaveWaitUntil")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    caps.did_save_provider = sync_obj
                        .get("save")
                        .map(|v| !v.is_null() && v != &Value::Bool(false))
                        .unwrap_or(false);
                }
                _ => {}
            }
        }

        caps.did_change_watched_files_provider = caps_obj
            .get("workspace")
            .and_then(|w| w.as_object())
            .and_then(|w| w.get("didChangeWatchedFiles"))
            .map(|v| !v.is_null())
            .unwrap_or(false);

        if !caps.workspace_configuration_provider {
            caps.workspace_configuration_provider = caps_obj
                .get("workspace")
                .and_then(|w| w.as_object())
                .and_then(|w| w.get("workspaceCapabilities"))
                .map(|v| !v.is_null())
                .unwrap_or(false);
        }

        caps.completion_item_resolve_provider = caps_obj
            .get("completionProvider")
            .and_then(|cp| cp.as_object())
            .and_then(|cp| cp.get("resolveProvider"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        caps.code_lens_resolve_provider = caps_obj
            .get("codeLensProvider")
            .and_then(|cl| cl.as_object())
            .and_then(|cl| cl.get("resolveProvider"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        caps.document_link_provider = caps_obj.get("documentLinkProvider").is_some()
            && !caps_obj["documentLinkProvider"].is_null();
        caps.document_link_resolve_provider = caps_obj
            .get("documentLinkProvider")
            .and_then(|dl| dl.as_object())
            .and_then(|dl| dl.get("resolveProvider"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        caps.color_provider = caps_obj
            .get("colorProvider")
            .map(|v| !v.is_null() && v != &Value::Bool(false))
            .unwrap_or(false);

        caps.declaration_provider = is_truthy(caps_obj.get("declarationProvider"));

        caps.type_hierarchy_provider = caps_obj
            .get("typeHierarchyProvider")
            .map(|v| !v.is_null() && v != &Value::Bool(false))
            .unwrap_or(false);

        caps.call_hierarchy_provider = caps_obj
            .get("callHierarchyProvider")
            .map(|v| !v.is_null() && v != &Value::Bool(false))
            .unwrap_or(false);

        caps.semantic_tokens_range_provider = caps_obj
            .get("semanticTokensProvider")
            .and_then(|st| st.as_object())
            .and_then(|st| st.get("range"))
            .map(|v| !v.is_null() && v != &Value::Bool(false))
            .unwrap_or(false);

        caps.inline_completion_provider = caps_obj
            .get("inlineCompletionProvider")
            .map(|v| !v.is_null() && v != &Value::Bool(false))
            .unwrap_or(false);

        caps
    }
}

fn is_truthy(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Bool(b)) => *b,
        Some(Value::Object(_)) => true,
        Some(Value::Null) => false,
        _ => false,
    }
}

pub struct Client {
    child: Child,
    next_id: i64,
    read_buf: Vec<u8>,
}

impl Client {
    pub fn init(server_path: &str, server_args: &[String]) -> std::io::Result<Self> {
        let mut cmd = Command::new(server_path);
        cmd.args(server_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let child = cmd.spawn()?;

        Ok(Client {
            child,
            next_id: 1,
            read_buf: Vec::new(),
        })
    }

    pub fn send_request(&mut self, method: &str, params: Option<Value>) -> std::io::Result<i64> {
        if let Ok(Some(status)) = self.child.try_wait() {
            let code = status.code().unwrap_or(-1);
            return Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                format!("LSP server exited unexpectedly with code {}", code),
            ));
        }

        let id = self.next_id;
        self.next_id += 1;

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        };

        let content = serde_json::to_string(&request)?;
        self.send_raw(&content)?;
        Ok(id)
    }

    pub fn send_notification(
        &mut self,
        method: &str,
        params: Option<Value>,
    ) -> std::io::Result<()> {
        if let Ok(Some(status)) = self.child.try_wait() {
            let code = status.code().unwrap_or(-1);
            return Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                format!("LSP server exited unexpectedly with code {}", code),
            ));
        }

        let notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
        };

        let content = serde_json::to_string(&notification)?;
        self.send_raw(&content)?;
        Ok(())
    }

    fn send_raw(&mut self, content: &str) -> std::io::Result<()> {
        if let Ok(Some(_)) = self.child.try_wait() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "LSP server process exited",
            ));
        }

        let stdin = self.child.stdin.as_mut().ok_or(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "No stdin",
        ))?;

        let header = format!("Content-Length: {}\r\n\r\n", content.len());

        stdin.write_all(header.as_bytes())?;
        stdin.write_all(content.as_bytes())?;
        stdin.flush()?;
        Ok(())
    }

    pub fn read_message(&mut self, timeout: Duration) -> std::io::Result<Option<Value>> {
        let deadline = Instant::now() + timeout;

        while Instant::now() < deadline {
            if let Some(msg) = self.try_parse_message()? {
                return Ok(Some(msg));
            }

            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Ok(None);
            }

            let process_exited = match self.child.try_wait() {
                Ok(Some(_)) => true,
                Ok(None) => false,
                Err(_) => false,
            };

            if let Some(stdout) = self.child.stdout.as_mut() {
                let mut tmp = [0u8; 4096];

                match stdout.read(&mut tmp) {
                    Ok(0) => {
                        return Ok(None);
                    }
                    Ok(n) => {
                        self.read_buf.extend_from_slice(&tmp[0..n]);
                        if process_exited {
                            continue;
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => {
                        if process_exited {
                            return Ok(None);
                        }
                        return Ok(None);
                    }
                }
            } else {
                return Ok(None);
            }
        }

        Ok(None)
    }

    fn try_parse_message(&mut self) -> std::io::Result<Option<Value>> {
        let data = &self.read_buf[..];
        let sep = match memmem::find(data, b"\r\n\r\n") {
            Some(pos) => pos,
            None => {
                return Ok(None);
            }
        };

        let header = String::from_utf8_lossy(&data[..sep]);
        let cl_start = header.find("Content-Length: ").ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "No Content-Length")
        })?;

        let cl_val_start = cl_start + 16; // "Content-Length: ".len()
        let cl_end = header[cl_val_start..]
            .find('\r')
            .map(|i| cl_val_start + i)
            .unwrap_or(header.len());

        let content_length: usize = header[cl_val_start..cl_end]
            .trim()
            .parse()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let body_start = sep + 4;
        if data.len() < body_start + content_length {
            return Ok(None);
        }

        let body = &data[body_start..body_start + content_length];
        let value: Value = serde_json::from_slice(body)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        self.read_buf.drain(..body_start + content_length);

        Ok(Some(value))
    }

    pub fn read_response(&mut self, id: i64, timeout: Duration) -> std::io::Result<Option<Value>> {
        let deadline = Instant::now() + timeout;

        while Instant::now() < deadline {
            let remaining = deadline.saturating_duration_since(Instant::now());
            let msg = match self.read_message(remaining)? {
                Some(m) => m,
                None => {
                    return Ok(None);
                }
            };

            let Some(obj) = msg.as_object() else {
                continue;
            };

            let Some(msg_id) = obj.get("id") else {
                continue;
            };

            let Some(msg_id_int) = msg_id.as_i64() else {
                continue;
            };

            if msg_id_int == id {
                return Ok(Some(msg));
            }
        }

        Ok(None)
    }

    pub fn deinit(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }

    pub fn is_alive(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(None) => true,
            _ => false,
        }
    }
}
