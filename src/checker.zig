const std = @import("std");
const lsp = @import("lsp.zig");

pub const CheckStatus = enum {
    passed,
    failed,
    skipped, // server doesn't advertise this capability
    timeout,
};

pub const CheckResult = struct {
    name: []const u8,
    method: []const u8,
    status: CheckStatus,
    detail: []const u8,
    duration_ms: i64,
};

pub const HealthChecker = struct {
    allocator: std.mem.Allocator,
    client: lsp.Client,
    capabilities: lsp.ServerCapabilities,
    results: std.ArrayList(CheckResult),

    const DOC_URI = "file:///tmp/lsp_health_check.zig";
    const DOC_CONTENT =
        \\const std = @import("std");
        \\
        \\pub fn add(a: i32, b: i32) i32 {
        \\    return a + b;
        \\}
        \\
        \\pub fn main() void {
        \\    const x = add(1, 2);
        \\    _ = x;
        \\}
        \\
    ;

    const TIMEOUT_MS = 5000;

    pub fn init(allocator: std.mem.Allocator, server_path: []const u8, server_args: []const []const u8) !HealthChecker {
        const client = try lsp.Client.init(allocator, server_path, server_args);
        const empty_cr_arraylist: std.ArrayList(CheckResult) = .empty;
        return HealthChecker{
            .allocator = allocator,
            .client = client,
            .capabilities = .{},
            .results = empty_cr_arraylist,
        };
    }

    pub fn deinit(self: *HealthChecker) void {
        self.client.deinit();
    }

    pub fn runAllChecks(self: *HealthChecker) ![]CheckResult {
        try self.checkInitialize();
        try self.checkDidOpen();
        try self.checkHover();
        try self.checkSignatureHelp();
        try self.checkCompletion();
        try self.checkDefinition();
        try self.checkReferences();
        try self.checkDocumentSymbol();
        try self.checkFormatting();
        try self.checkCodeAction();
        try self.checkRename();
        try self.checkInlayHint();
        try self.checkWorkspaceSymbol();
        try self.checkShutdown();
        return self.results.items;
    }

    fn record(self: *HealthChecker, name: []const u8, method: []const u8, status: CheckStatus, detail: []const u8, duration_ms: i64) !void {
        try self.results.append(std.heap.page_allocator, .{
            .name = name,
            .method = method,
            .status = status,
            .detail = detail,
            .duration_ms = duration_ms,
        });
    }

    fn checkInitialize(self: *HealthChecker) !void {
        const t0 = std.time.milliTimestamp();

        const params = try self.buildInitializeParams();
        const id = try self.client.sendRequest("initialize", params);

        var resp = try self.client.readResponse(id, TIMEOUT_MS) orelse {
            try self.record("Initialize", "initialize", .timeout, "no response", std.time.milliTimestamp() - t0);
            return;
        };
        defer resp.deinit();

        const dt = std.time.milliTimestamp() - t0;

        if (resp.hasError()) {
            try self.record("Initialize", "initialize", .failed, "server returned error", dt);
            return;
        }

        if (resp.getResult()) |result| {
            self.capabilities = lsp.parseServerCapabilities(result);
        }

        // Send initialized notification
        try self.client.sendNotification("initialized", try self.jsonObject(&.{}));

        try self.record("Initialize", "initialize", .passed, "handshake complete", dt);
    }

    fn checkDidOpen(self: *HealthChecker) !void {
        const t0 = std.time.milliTimestamp();

        const params = try self.jsonObject(&.{
            .{ "textDocument", try self.jsonObject(&.{
                .{ "uri", try self.jsonString(DOC_URI) },
                .{ "languageId", try self.jsonString("zig") },
                .{ "version", std.json.Value{ .integer = 1 } },
                .{ "text", try self.jsonString(DOC_CONTENT) },
            }) },
        });

        try self.client.sendNotification("textDocument/didOpen", params);

        // Small delay to let server process
        std.Thread.sleep(200 * std.time.ns_per_ms);

        const dt = std.time.milliTimestamp() - t0;
        try self.record("Open Document", "textDocument/didOpen", .passed, "notification sent", dt);
    }

    fn checkHover(self: *HealthChecker) !void {
        if (!self.capabilities.hover_provider) {
            try self.record("Hover", "textDocument/hover", .skipped, "not advertised", 0);
            return;
        }

        const t0 = std.time.milliTimestamp();
        const params = try self.textDocumentPosition(2, 7); // on 'add' function name
        const id = try self.client.sendRequest("textDocument/hover", params);

        var resp = try self.client.readResponse(id, TIMEOUT_MS) orelse {
            try self.record("Hover", "textDocument/hover", .timeout, "no response", std.time.milliTimestamp() - t0);
            return;
        };
        defer resp.deinit();

        const dt = std.time.milliTimestamp() - t0;
        if (resp.hasError()) {
            try self.record("Hover", "textDocument/hover", .failed, "server error", dt);
        } else {
            try self.record("Hover", "textDocument/hover", .passed, "response received", dt);
        }
    }

    fn checkSignatureHelp(self: *HealthChecker) !void {
        if (!self.capabilities.signature_help_provider) {
            try self.record("Signature Help", "textDocument/signatureHelp", .skipped, "not advertised", 0);
            return;
        }

        const t0 = std.time.milliTimestamp();
        const params = try self.textDocumentPosition(8, 19); // inside add(
        const id = try self.client.sendRequest("textDocument/signatureHelp", params);

        var resp = try self.client.readResponse(id, TIMEOUT_MS) orelse {
            try self.record("Signature Help", "textDocument/signatureHelp", .timeout, "no response", std.time.milliTimestamp() - t0);
            return;
        };
        defer resp.deinit();

        const dt = std.time.milliTimestamp() - t0;
        if (resp.hasError()) {
            try self.record("Signature Help", "textDocument/signatureHelp", .failed, "server error", dt);
        } else {
            try self.record("Signature Help", "textDocument/signatureHelp", .passed, "response received", dt);
        }
    }

    fn checkCompletion(self: *HealthChecker) !void {
        if (!self.capabilities.completion_provider) {
            try self.record("Completion", "textDocument/completion", .skipped, "not advertised", 0);
            return;
        }

        const t0 = std.time.milliTimestamp();
        const params = try self.textDocumentPosition(8, 14); // after 'const x = '
        const id = try self.client.sendRequest("textDocument/completion", params);

        var resp = try self.client.readResponse(id, TIMEOUT_MS) orelse {
            try self.record("Completion", "textDocument/completion", .timeout, "no response", std.time.milliTimestamp() - t0);
            return;
        };
        defer resp.deinit();

        const dt = std.time.milliTimestamp() - t0;
        if (resp.hasError()) {
            try self.record("Completion", "textDocument/completion", .failed, "server error", dt);
        } else {
            try self.record("Completion", "textDocument/completion", .passed, "response received", dt);
        }
    }

    fn checkDefinition(self: *HealthChecker) !void {
        if (!self.capabilities.definition_provider) {
            try self.record("Go to Definition", "textDocument/definition", .skipped, "not advertised", 0);
            return;
        }

        const t0 = std.time.milliTimestamp();
        const params = try self.textDocumentPosition(8, 18); // on 'add' call
        const id = try self.client.sendRequest("textDocument/definition", params);

        var resp = try self.client.readResponse(id, TIMEOUT_MS) orelse {
            try self.record("Go to Definition", "textDocument/definition", .timeout, "no response", std.time.milliTimestamp() - t0);
            return;
        };
        defer resp.deinit();

        const dt = std.time.milliTimestamp() - t0;
        if (resp.hasError()) {
            try self.record("Go to Definition", "textDocument/definition", .failed, "server error", dt);
        } else {
            try self.record("Go to Definition", "textDocument/definition", .passed, "response received", dt);
        }
    }

    fn checkReferences(self: *HealthChecker) !void {
        if (!self.capabilities.references_provider) {
            try self.record("Find References", "textDocument/references", .skipped, "not advertised", 0);
            return;
        }

        const t0 = std.time.milliTimestamp();

        const base = try self.textDocumentPosition(2, 7);
        var obj = base.object;
        try obj.put("context", try self.jsonObject(&.{
            .{ "includeDeclaration", std.json.Value{ .bool = true } },
        }));
        const params = std.json.Value{ .object = obj };

        const id = try self.client.sendRequest("textDocument/references", params);

        var resp = try self.client.readResponse(id, TIMEOUT_MS) orelse {
            try self.record("Find References", "textDocument/references", .timeout, "no response", std.time.milliTimestamp() - t0);
            return;
        };
        defer resp.deinit();

        const dt = std.time.milliTimestamp() - t0;
        if (resp.hasError()) {
            try self.record("Find References", "textDocument/references", .failed, "server error", dt);
        } else {
            try self.record("Find References", "textDocument/references", .passed, "response received", dt);
        }
    }

    fn checkDocumentSymbol(self: *HealthChecker) !void {
        if (!self.capabilities.document_symbol_provider) {
            try self.record("Document Symbols", "textDocument/documentSymbol", .skipped, "not advertised", 0);
            return;
        }

        const t0 = std.time.milliTimestamp();
        const params = try self.jsonObject(&.{
            .{ "textDocument", try self.textDocumentIdentifier() },
        });
        const id = try self.client.sendRequest("textDocument/documentSymbol", params);

        var resp = try self.client.readResponse(id, TIMEOUT_MS) orelse {
            try self.record("Document Symbols", "textDocument/documentSymbol", .timeout, "no response", std.time.milliTimestamp() - t0);
            return;
        };
        defer resp.deinit();

        const dt = std.time.milliTimestamp() - t0;
        if (resp.hasError()) {
            try self.record("Document Symbols", "textDocument/documentSymbol", .failed, "server error", dt);
        } else {
            try self.record("Document Symbols", "textDocument/documentSymbol", .passed, "response received", dt);
        }
    }

    fn checkFormatting(self: *HealthChecker) !void {
        if (!self.capabilities.document_formatting_provider) {
            try self.record("Formatting", "textDocument/formatting", .skipped, "not advertised", 0);
            return;
        }

        const t0 = std.time.milliTimestamp();
        const params = try self.jsonObject(&.{
            .{ "textDocument", try self.textDocumentIdentifier() },
            .{ "options", try self.jsonObject(&.{
                .{ "tabSize", std.json.Value{ .integer = 4 } },
                .{ "insertSpaces", std.json.Value{ .bool = true } },
            }) },
        });
        const id = try self.client.sendRequest("textDocument/formatting", params);

        var resp = try self.client.readResponse(id, TIMEOUT_MS) orelse {
            try self.record("Formatting", "textDocument/formatting", .timeout, "no response", std.time.milliTimestamp() - t0);
            return;
        };
        defer resp.deinit();

        const dt = std.time.milliTimestamp() - t0;
        if (resp.hasError()) {
            try self.record("Formatting", "textDocument/formatting", .failed, "server error", dt);
        } else {
            try self.record("Formatting", "textDocument/formatting", .passed, "response received", dt);
        }
    }

    fn checkCodeAction(self: *HealthChecker) !void {
        if (!self.capabilities.code_action_provider) {
            try self.record("Code Actions", "textDocument/codeAction", .skipped, "not advertised", 0);
            return;
        }

        const t0 = std.time.milliTimestamp();
        const params = try self.jsonObject(&.{
            .{ "textDocument", try self.textDocumentIdentifier() },
            .{ "range", try self.makeRange(0, 0, 0, 10) },
            .{ "context", try self.jsonObject(&.{
                .{ "diagnostics", std.json.Value{ .array = std.json.Array.init(self.allocator) } },
            }) },
        });
        const id = try self.client.sendRequest("textDocument/codeAction", params);

        var resp = try self.client.readResponse(id, TIMEOUT_MS) orelse {
            try self.record("Code Actions", "textDocument/codeAction", .timeout, "no response", std.time.milliTimestamp() - t0);
            return;
        };
        defer resp.deinit();

        const dt = std.time.milliTimestamp() - t0;
        if (resp.hasError()) {
            try self.record("Code Actions", "textDocument/codeAction", .failed, "server error", dt);
        } else {
            try self.record("Code Actions", "textDocument/codeAction", .passed, "response received", dt);
        }
    }

    fn checkRename(self: *HealthChecker) !void {
        if (!self.capabilities.rename_provider) {
            try self.record("Rename Symbol", "textDocument/rename", .skipped, "not advertised", 0);
            return;
        }

        const t0 = std.time.milliTimestamp();
        const base = try self.textDocumentPosition(2, 7);
        var obj = base.object;
        try obj.put("newName", try self.jsonString("sum"));
        const params = std.json.Value{ .object = obj };
        const id = try self.client.sendRequest("textDocument/rename", params);

        var resp = try self.client.readResponse(id, TIMEOUT_MS) orelse {
            try self.record("Rename Symbol", "textDocument/rename", .timeout, "no response", std.time.milliTimestamp() - t0);
            return;
        };
        defer resp.deinit();

        const dt = std.time.milliTimestamp() - t0;
        if (resp.hasError()) {
            try self.record("Rename Symbol", "textDocument/rename", .failed, "server error", dt);
        } else {
            try self.record("Rename Symbol", "textDocument/rename", .passed, "response received", dt);
        }
    }

    fn checkInlayHint(self: *HealthChecker) !void {
        if (!self.capabilities.inlay_hint_provider) {
            try self.record("Inlay Hints", "textDocument/inlayHint", .skipped, "not advertised", 0);
            return;
        }

        const t0 = std.time.milliTimestamp();
        const params = try self.jsonObject(&.{
            .{ "textDocument", try self.textDocumentIdentifier() },
            .{ "range", try self.makeRange(0, 0, 10, 0) },
        });
        const id = try self.client.sendRequest("textDocument/inlayHint", params);

        var resp = try self.client.readResponse(id, TIMEOUT_MS) orelse {
            try self.record("Inlay Hints", "textDocument/inlayHint", .timeout, "no response", std.time.milliTimestamp() - t0);
            return;
        };
        defer resp.deinit();

        const dt = std.time.milliTimestamp() - t0;
        if (resp.hasError()) {
            try self.record("Inlay Hints", "textDocument/inlayHint", .failed, "server error", dt);
        } else {
            try self.record("Inlay Hints", "textDocument/inlayHint", .passed, "response received", dt);
        }
    }

    fn checkWorkspaceSymbol(self: *HealthChecker) !void {
        if (!self.capabilities.workspace_symbol_provider) {
            try self.record("Workspace Symbols", "workspace/symbol", .skipped, "not advertised", 0);
            return;
        }

        const t0 = std.time.milliTimestamp();
        const params = try self.jsonObject(&.{
            .{ "query", try self.jsonString("add") },
        });
        const id = try self.client.sendRequest("workspace/symbol", params);

        var resp = try self.client.readResponse(id, TIMEOUT_MS) orelse {
            try self.record("Workspace Symbols", "workspace/symbol", .timeout, "no response", std.time.milliTimestamp() - t0);
            return;
        };
        defer resp.deinit();

        const dt = std.time.milliTimestamp() - t0;
        if (resp.hasError()) {
            try self.record("Workspace Symbols", "workspace/symbol", .failed, "server error", dt);
        } else {
            try self.record("Workspace Symbols", "workspace/symbol", .passed, "response received", dt);
        }
    }

    fn checkShutdown(self: *HealthChecker) !void {
        const t0 = std.time.milliTimestamp();

        const id = try self.client.sendRequest("shutdown", null);
        var resp = try self.client.readResponse(id, TIMEOUT_MS) orelse {
            try self.record("Shutdown", "shutdown/exit", .timeout, "no response", std.time.milliTimestamp() - t0);
            return;
        };
        defer resp.deinit();

        try self.client.sendNotification("exit", null);

        const dt = std.time.milliTimestamp() - t0;
        if (resp.hasError()) {
            try self.record("Shutdown", "shutdown/exit", .failed, "server error", dt);
        } else {
            try self.record("Shutdown", "shutdown/exit", .passed, "clean exit", dt);
        }
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Helpers
    // ──────────────────────────────────────────────────────────────────────────

    fn buildInitializeParams(self: *HealthChecker) !std.json.Value {
        return self.jsonObject(&.{
            .{ "processId", std.json.Value{ .integer = std.os.linux.getpid() } },
            .{ "clientInfo", try self.jsonObject(&.{
                .{ "name", try self.jsonString("lsp-health-checker") },
                .{ "version", try self.jsonString("1.0.0") },
            }) },
            .{ "rootUri", try self.jsonString("file:///tmp") },
            .{ "capabilities", try self.jsonObject(&.{
                .{ "textDocument", try self.jsonObject(&.{
                    .{ "hover", try self.jsonObject(&.{
                        .{ "contentFormat", try self.jsonStringArray(&.{ "markdown", "plaintext" }) },
                    }) },
                    .{ "signatureHelp", try self.jsonObject(&.{
                        .{ "signatureInformation", try self.jsonObject(&.{
                            .{ "documentationFormat", try self.jsonStringArray(&.{"markdown"}) },
                        }) },
                    }) },
                    .{ "completion", try self.jsonObject(&.{
                        .{ "completionItem", try self.jsonObject(&.{
                            .{ "snippetSupport", std.json.Value{ .bool = true } },
                        }) },
                    }) },
                    .{ "definition", try self.jsonObject(&.{
                        .{ "dynamicRegistration", std.json.Value{ .bool = false } },
                    }) },
                    .{ "references", try self.jsonObject(&.{
                        .{ "dynamicRegistration", std.json.Value{ .bool = false } },
                    }) },
                    .{ "documentSymbol", try self.jsonObject(&.{
                        .{ "hierarchicalDocumentSymbolSupport", std.json.Value{ .bool = true } },
                    }) },
                    .{ "formatting", try self.jsonObject(&.{
                        .{ "dynamicRegistration", std.json.Value{ .bool = false } },
                    }) },
                    .{ "codeAction", try self.jsonObject(&.{
                        .{ "dynamicRegistration", std.json.Value{ .bool = false } },
                    }) },
                    .{ "rename", try self.jsonObject(&.{
                        .{ "dynamicRegistration", std.json.Value{ .bool = false } },
                    }) },
                    .{ "inlayHint", try self.jsonObject(&.{
                        .{ "dynamicRegistration", std.json.Value{ .bool = false } },
                    }) },
                }) },
                .{ "workspace", try self.jsonObject(&.{
                    .{ "symbol", try self.jsonObject(&.{
                        .{ "dynamicRegistration", std.json.Value{ .bool = false } },
                    }) },
                }) },
            }) },
            .{ "trace", try self.jsonString("off") },
            .{ "workspaceFolders", std.json.Value{ .null = {} } },
        });
    }

    fn textDocumentIdentifier(self: *HealthChecker) !std.json.Value {
        return self.jsonObject(&.{
            .{ "uri", try self.jsonString(DOC_URI) },
        });
    }

    fn textDocumentPosition(self: *HealthChecker, line: i64, character: i64) !std.json.Value {
        return self.jsonObject(&.{
            .{ "textDocument", try self.textDocumentIdentifier() },
            .{ "position", try self.makePosition(line, character) },
        });
    }

    fn makePosition(self: *HealthChecker, line: i64, character: i64) !std.json.Value {
        return self.jsonObject(&.{
            .{ "line", std.json.Value{ .integer = line } },
            .{ "character", std.json.Value{ .integer = character } },
        });
    }

    fn makeRange(self: *HealthChecker, start_line: i64, start_char: i64, end_line: i64, end_char: i64) !std.json.Value {
        return self.jsonObject(&.{
            .{ "start", try self.makePosition(start_line, start_char) },
            .{ "end", try self.makePosition(end_line, end_char) },
        });
    }

    fn jsonString(self: *HealthChecker, s: []const u8) !std.json.Value {
        _ = self;
        return std.json.Value{ .string = s };
    }

    fn jsonStringArray(self: *HealthChecker, strings: []const []const u8) !std.json.Value {
        var arr = std.json.Array.init(self.allocator);
        for (strings) |s| {
            try arr.append(std.json.Value{ .string = s });
        }
        return std.json.Value{ .array = arr };
    }

    fn jsonObject(self: *HealthChecker, fields: []const struct { []const u8, std.json.Value }) !std.json.Value {
        var obj = std.json.ObjectMap.init(self.allocator);
        for (fields) |field| {
            try obj.put(field[0], field[1]);
        }
        return std.json.Value{ .object = obj };
    }
};
