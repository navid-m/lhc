const std = @import("std");

pub const JsonRpcRequest = struct {
    jsonrpc: []const u8 = "2.0",
    id: ?i64,
    method: []const u8,
    params: ?std.json.Value,
};

pub const JsonRpcNotification = struct {
    jsonrpc: []const u8 = "2.0",
    method: []const u8,
    params: ?std.json.Value,
};

pub const JsonRpcResponse = struct {
    id: ?i64,
    result: ?std.json.Value,
    @"error": ?ResponseError,
};

pub const ResponseError = struct {
    code: i64,
    message: []const u8,
};

pub const ServerCapabilities = struct {
    hover_provider: bool = false,
    signature_help_provider: bool = false,
    completion_provider: bool = false,
    definition_provider: bool = false,
    references_provider: bool = false,
    document_symbol_provider: bool = false,
    document_formatting_provider: bool = false,
    code_action_provider: bool = false,
    rename_provider: bool = false,
    inlay_hint_provider: bool = false,
    workspace_symbol_provider: bool = false,
};

/// Parse server capabilities from the initialize response
pub fn parseServerCapabilities(value: std.json.Value) ServerCapabilities {
    var caps = ServerCapabilities{};

    const obj = switch (value) {
        .object => |o| o,
        else => return caps,
    };

    const capabilities = obj.get("capabilities") orelse return caps;
    const caps_obj = switch (capabilities) {
        .object => |o| o,
        else => return caps,
    };

    caps.hover_provider = isTruthy(caps_obj.get("hoverProvider"));
    caps.signature_help_provider = caps_obj.get("signatureHelpProvider") != null;
    caps.completion_provider = caps_obj.get("completionProvider") != null;
    caps.definition_provider = isTruthy(caps_obj.get("definitionProvider"));
    caps.references_provider = isTruthy(caps_obj.get("referencesProvider"));
    caps.document_symbol_provider = isTruthy(caps_obj.get("documentSymbolProvider"));
    caps.document_formatting_provider = isTruthy(caps_obj.get("documentFormattingProvider"));
    caps.code_action_provider = isTruthy(caps_obj.get("codeActionProvider"));
    caps.rename_provider = isTruthy(caps_obj.get("renameProvider"));
    caps.inlay_hint_provider = isTruthy(caps_obj.get("inlayHintProvider"));

    if (caps_obj.get("workspace")) |ws| {
        if (ws == .object) {
            if (ws.object.get("symbol")) |sym| {
                caps.workspace_symbol_provider = sym != .null;
            }
        }
    }

    return caps;
}

fn isTruthy(v: ?std.json.Value) bool {
    const val = v orelse return false;
    return switch (val) {
        .bool => |b| b,
        .object => true,
        .null => false,
        else => false,
    };
}

/// LSP Client manages the subprocess and read/write buffers
pub const Client = struct {
    allocator: std.mem.Allocator,
    child: std.process.Child,
    next_id: i64,
    read_buf: std.ArrayList(u8),

    pub fn init(allocator: std.mem.Allocator, server_path: []const u8, server_args: []const []const u8) !Client {
        var argv: std.ArrayList([]const u8) = .empty;
        defer argv.deinit(allocator);
        try argv.append(allocator, server_path);
        for (server_args) |arg| try argv.append(allocator, arg);

        var child = std.process.Child.init(argv.items, allocator);
        child.stdin_behavior = .Pipe;
        child.stdout_behavior = .Pipe;
        child.stderr_behavior = .Ignore;

        try child.spawn();

        const empty_arraylist: std.ArrayList(u8) = .empty;
        return Client{
            .allocator = allocator,
            .child = child,
            .next_id = 1,
            .read_buf = empty_arraylist,
        };
    }

    pub fn deinit(self: *Client) void {
        // Send shutdown/exit before killing
        if (self.child.stdin) |stdin| {
            _ = stdin;
        }
        _ = self.child.kill() catch {};
        self.read_buf.deinit(std.heap.page_allocator);
    }

    pub fn sendRequest(self: *Client, method: []const u8, params: ?std.json.Value) !i64 {
        const id = self.next_id;
        self.next_id += 1;

        var buf: std.ArrayList(u8) = .empty;
        defer buf.deinit(std.heap.page_allocator);

        const writer = buf.writer(std.heap.page_allocator);
        try writer.writeAll("{\"jsonrpc\":\"2.0\",\"id\":");
        try writer.writeAll(try std.fmt.allocPrint(std.heap.page_allocator, "{d}", .{id}));
        try writer.writeAll(",\"method\":\"");
        try writer.writeAll(method);
        try writer.writeAll("\"");

        if (params) |p| {
            try writer.writeAll(",\"params\":");
            const fmter = std.json.fmt(p, .{});
            try writer.writeAll(try std.fmt.allocPrint(std.heap.page_allocator, "{f}", .{fmter}));
        } else {
            try writer.writeAll(",\"params\":null");
        }
        try writer.writeAll("}");

        try self.sendRaw(buf.items);
        return id;
    }

    pub fn sendNotification(self: *Client, method: []const u8, params: ?std.json.Value) !void {
        var buf: std.ArrayList(u8) = .empty;
        defer buf.deinit(std.heap.page_allocator);

        var writer = buf.writer(std.heap.page_allocator);

        try writer.print("{{\"jsonrpc\":\"2.0\",\"method\":\"{s}\"", .{method});

        if (params) |p| {
            try writer.print(",\"params\":{f}", .{std.json.fmt(p, .{})});
        }

        try writer.writeAll("}");

        try self.sendRaw(buf.items);
    }

    fn sendRaw(self: *Client, content: []const u8) !void {
        const stdin = self.child.stdin orelse return error.NoStdin;
        var header_buf: [64]u8 = undefined;
        const header = try std.fmt.bufPrint(&header_buf, "Content-Length: {d}\r\n\r\n", .{content.len});
        try stdin.writeAll(header);
        try stdin.writeAll(content);
    }

    /// Read one JSON-RPC message, parse it. Caller owns the returned value's arena.
    pub fn readMessage(self: *Client, timeout_ms: u64) !?ParsedResponse {
        const stdout = self.child.stdout orelse return error.NoStdout;

        const deadline = std.time.milliTimestamp() + @as(i64, @intCast(timeout_ms));

        // Read until we have a complete message
        while (true) {
            if (std.time.milliTimestamp() > deadline) return null;

            // Try to parse existing buffer
            if (try self.tryParseMessage()) |msg| return msg;

            const remaining_ms = deadline - std.time.milliTimestamp();
            if (remaining_ms <= 0) return null;

            var tmp: [4096]u8 = undefined;
            const n = stdout.read(&tmp) catch return null;
            if (n == 0) return null;
            try self.read_buf.appendSlice(std.heap.page_allocator, tmp[0..n]);
        }
    }

    fn tryParseMessage(self: *Client) !?ParsedResponse {
        const data = self.read_buf.items;

        // Find the header/body separator
        const sep = std.mem.indexOf(u8, data, "\r\n\r\n") orelse return null;
        const header = data[0..sep];

        // Parse Content-Length
        const cl_prefix = "Content-Length: ";
        const cl_start = std.mem.indexOf(u8, header, cl_prefix) orelse return null;
        const cl_val_start = cl_start + cl_prefix.len;
        const cl_end = std.mem.indexOfScalarPos(u8, header, cl_val_start, '\r') orelse header.len;
        const content_length = try std.fmt.parseInt(usize, std.mem.trim(u8, header[cl_val_start..cl_end], " \r\n"), 10);

        const body_start = sep + 4;
        if (data.len < body_start + content_length) return null;

        const body = data[body_start .. body_start + content_length];

        // Parse JSON
        var arena = std.heap.ArenaAllocator.init(self.allocator);
        errdefer arena.deinit();
        const parsed = try std.json.parseFromSliceLeaky(std.json.Value, arena.allocator(), body, .{});

        // Consume the bytes
        const consumed = body_start + content_length;
        try self.read_buf.replaceRange(std.heap.page_allocator, 0, consumed, &.{});

        return ParsedResponse{ .arena = arena, .value = parsed };
    }

    /// Read the response to a specific request ID, skipping notifications
    pub fn readResponse(self: *Client, id: i64, timeout_ms: u64) !?ParsedResponse {
        const deadline = std.time.milliTimestamp() + @as(i64, @intCast(timeout_ms));

        while (std.time.milliTimestamp() < deadline) {
            const remaining = @as(u64, @intCast(@max(0, deadline - std.time.milliTimestamp())));
            const msg = try self.readMessage(@min(remaining, 200)) orelse return null;

            // Check if this is the response we want
            const obj = switch (msg.value) {
                .object => |o| o,
                else => {
                    msg.arena.deinit();
                    continue;
                },
            };

            // Skip notifications (no id field or id is null)
            const msg_id = obj.get("id") orelse {
                msg.arena.deinit();
                continue;
            };

            const msg_id_int = switch (msg_id) {
                .integer => |i| i,
                else => {
                    msg.arena.deinit();
                    continue;
                },
            };

            if (msg_id_int == id) return msg;
            msg.arena.deinit();
        }
        return null;
    }
};

pub const ParsedResponse = struct {
    arena: std.heap.ArenaAllocator,
    value: std.json.Value,

    pub fn deinit(self: *ParsedResponse) void {
        self.arena.deinit();
    }

    pub fn hasError(self: *const ParsedResponse) bool {
        const obj = switch (self.value) {
            .object => |o| o,
            else => return false,
        };
        return obj.get("error") != null;
    }

    pub fn getResult(self: *const ParsedResponse) ?std.json.Value {
        const obj = switch (self.value) {
            .object => |o| o,
            else => return null,
        };
        return obj.get("result");
    }
};
