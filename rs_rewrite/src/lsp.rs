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

#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcResponse {
    pub id: Option<i64>,
    pub result: Option<Value>,
    pub error: Option<ResponseError>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResponseError {
    pub code: i64,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct ServerCapabilities {
    pub hover_provider: bool,
    pub signature_help_provider: bool,
    pub completion_provider: bool,
    pub definition_provider: bool,
    pub references_provider: bool,
    pub document_symbol_provider: bool,
    pub document_formatting_provider: bool,
    pub code_action_provider: bool,
    pub rename_provider: bool,
    pub inlay_hint_provider: bool,
    pub workspace_symbol_provider: bool,
}

impl ServerCapabilities {
    pub fn from_value(value: &Value) -> Self {
        let mut caps = ServerCapabilities::default();

        let Some(obj) = value.as_object() else {
            return caps;
        };

        let Some(capabilities) = obj.get("capabilities") else {
            return caps;
        };

        let Some(caps_obj) = capabilities.as_object() else {
            return caps;
        };

        caps.hover_provider = is_truthy(caps_obj.get("hoverProvider"));
        caps.signature_help_provider = caps_obj.get("signatureHelpProvider").is_some();
        caps.completion_provider = caps_obj.get("completionProvider").is_some();
        caps.definition_provider = is_truthy(caps_obj.get("definitionProvider"));
        caps.references_provider = is_truthy(caps_obj.get("referencesProvider"));
        caps.document_symbol_provider = is_truthy(caps_obj.get("documentSymbolProvider"));
        caps.document_formatting_provider = is_truthy(caps_obj.get("documentFormattingProvider"));
        caps.code_action_provider = is_truthy(caps_obj.get("codeActionProvider"));
        caps.rename_provider = is_truthy(caps_obj.get("renameProvider"));
        caps.inlay_hint_provider = is_truthy(caps_obj.get("inlayHintProvider"));

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
        let mut cmd = Command::new(server_path);
        cmd.args(server_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        let child = cmd.spawn()?;

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

    pub fn send_notification(&mut self, method: &str, params: Option<Value>) -> std::io::Result<()> {
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

            // Read more data
            if let Some(stdout) = self.child.stdout.as_mut() {
                let mut tmp = [0u8; 4096];
                
                match stdout.read(&mut tmp) {
                    Ok(0) => return Ok(None),
                    Ok(n) => {
                        self.read_buf.extend_from_slice(&tmp[0..n]);
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => return Ok(None),
                }
            } else {
                return Ok(None);
            }
        }

        Ok(None)
    }

    fn try_parse_message(&mut self) -> std::io::Result<Option<Value>> {
        let data = &self.read_buf[..];

        // Find the header/body separator
        let sep = match memmem::find(data, b"\r\n\r\n") {
            Some(pos) => pos,
            None => return Ok(None),
        };

        let header = String::from_utf8_lossy(&data[..sep]);

        // Parse Content-Length
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

        // Consume the bytes
        self.read_buf.drain(..body_start + content_length);

        Ok(Some(value))
    }

    pub fn read_response(
        &mut self,
        id: i64,
        timeout: Duration,
    ) -> std::io::Result<Option<Value>> {
        let deadline = Instant::now() + timeout;

        while Instant::now() < deadline {
            let remaining = deadline.saturating_duration_since(Instant::now());
            let msg = match self.read_message(remaining)? {
                Some(m) => m,
                None => return Ok(None),
            };

            // Check if this is the response we want
            let Some(obj) = msg.as_object() else {
                continue;
            };

            // Skip notifications (no id field or id is null)
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
}
