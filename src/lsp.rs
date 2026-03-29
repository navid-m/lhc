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
            }
        }

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
        eprintln!(
            "[DEBUG] Spawning LSP server: {} with args: {:?}",
            server_path, server_args
        );
        let mut cmd = Command::new(server_path);
        cmd.args(server_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let child = cmd.spawn()?;
        eprintln!("[DEBUG] LSP server spawned successfully");

        Ok(Client {
            child,
            next_id: 1,
            read_buf: Vec::new(),
        })
    }

    pub fn send_request(&mut self, method: &str, params: Option<Value>) -> std::io::Result<i64> {
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
        eprintln!("[DEBUG] Sending message: {}", content);
        eprintln!("[DEBUG] Message length: {} bytes", content.len());
        
        let stdin = self.child.stdin.as_mut().ok_or(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "No stdin",
        ))?;

        let header = format!("Content-Length: {}\r\n\r\n", content.len());
        eprintln!("[DEBUG] Writing header: {}", header.trim());
        eprintln!("[DEBUG] Header bytes: {:?}", header.as_bytes());
        
        stdin.write_all(header.as_bytes())?;
        stdin.write_all(content.as_bytes())?;
        stdin.flush()?;
        eprintln!("[DEBUG] Message sent and flushed");
        Ok(())
    }

    pub fn read_message(&mut self, timeout: Duration) -> std::io::Result<Option<Value>> {
        let deadline = Instant::now() + timeout;
        eprintln!("[DEBUG] read_message called with timeout: {:?}", timeout);

        while Instant::now() < deadline {
            if let Some(msg) = self.try_parse_message()? {
                return Ok(Some(msg));
            }

            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                eprintln!("[DEBUG] Timeout reached in read_message");
                return Ok(None);
            }

            // Check if process is still alive before reading
            let process_exited = match self.child.try_wait() {
                Ok(Some(status)) => {
                    eprintln!("[DEBUG] Process exited with status: {}", status);
                    true
                }
                Ok(None) => false,
                Err(e) => {
                    eprintln!("[DEBUG] Error checking process status: {}", e);
                    false
                }
            };

            // Read more data
            if let Some(stdout) = self.child.stdout.as_mut() {
                let mut tmp = [0u8; 4096];

                match stdout.read(&mut tmp) {
                    Ok(0) => {
                        eprintln!("[DEBUG] Read 0 bytes - pipe closed");
                        return Ok(None);
                    }
                    Ok(n) => {
                        eprintln!("[DEBUG] Read {} bytes from stdout", n);
                        self.read_buf.extend_from_slice(&tmp[0..n]);
                        
                        // If process exited but we got data, continue parsing
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
                    Err(e) => {
                        eprintln!("[DEBUG] Read error: {}", e);
                        if process_exited {
                            // Process exited and we can't read more, return what we have
                            return Ok(None);
                        }
                        return Ok(None);
                    }
                }
            } else {
                eprintln!("[DEBUG] No stdout available");
                return Ok(None);
            }
        }

        Ok(None)
    }

    fn try_parse_message(&mut self) -> std::io::Result<Option<Value>> {
        let data = &self.read_buf[..];
        eprintln!("[DEBUG] Buffer size: {} bytes", data.len());

        let sep = match memmem::find(data, b"\r\n\r\n") {
            Some(pos) => {
                eprintln!("[DEBUG] Found header separator at position {}", pos);
                pos
            }
            None => {
                eprintln!("[DEBUG] No header separator found yet");
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

        eprintln!("[DEBUG] Content-Length: {}", content_length);

        let body_start = sep + 4;
        if data.len() < body_start + content_length {
            eprintln!(
                "[DEBUG] Incomplete message: have {} bytes, need {}",
                data.len(),
                body_start + content_length
            );
            return Ok(None);
        }

        let body = &data[body_start..body_start + content_length];
        eprintln!("[DEBUG] Parsing JSON body...");
        let value: Value = serde_json::from_slice(body).map_err(|e| {
            eprintln!("[DEBUG] JSON parse error: {}", e);
            eprintln!("[DEBUG] Body was: {}", String::from_utf8_lossy(body));
            std::io::Error::new(std::io::ErrorKind::InvalidData, e)
        })?;

        self.read_buf.drain(..body_start + content_length);
        eprintln!("[DEBUG] Successfully parsed message");

        Ok(Some(value))
    }

    pub fn read_response(&mut self, id: i64, timeout: Duration) -> std::io::Result<Option<Value>> {
        let deadline = Instant::now() + timeout;

        while Instant::now() < deadline {
            let remaining = deadline.saturating_duration_since(Instant::now());
            let msg = match self.read_message(remaining)? {
                Some(m) => {
                    eprintln!(
                        "[DEBUG] read_response received message: {}",
                        serde_json::to_string(&m).unwrap_or_else(|_| "invalid json".to_string())
                    );
                    m
                }
                None => {
                    eprintln!("[DEBUG] read_response got None");
                    return Ok(None);
                }
            };

            let Some(obj) = msg.as_object() else {
                eprintln!("[DEBUG] Message is not an object");
                continue;
            };

            let Some(msg_id) = obj.get("id") else {
                eprintln!("[DEBUG] Message has no id field (likely a notification)");
                continue;
            };

            let Some(msg_id_int) = msg_id.as_i64() else {
                eprintln!("[DEBUG] Message id is not an integer");
                continue;
            };

            eprintln!("[DEBUG] Message id: {}, looking for: {}", msg_id_int, id);
            if msg_id_int == id {
                eprintln!("[DEBUG] Found matching response!");
                return Ok(Some(msg));
            }
        }

        eprintln!("[DEBUG] Timeout waiting for response id: {}", id);
        Ok(None)
    }

    pub fn deinit(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
