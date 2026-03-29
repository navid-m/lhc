const std = @import("std");
const lsp = @import("lsp.zig");
const checker = @import("checker.zig");
const display = @import("display.zig");
const builtin = @import("builtin");

pub fn main() !void {
    if (builtin.os.tag == .windows) {
        const windows = @import("windows.zig");
        const success = windows.SetConsoleOutputCP(65001);
        if (success == 0) {
            return error.FailedToSetCodePage;
        }
    }

    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const args = try std.process.argsAlloc(allocator);
    defer std.process.argsFree(allocator, args);

    if (args.len < 2) {
        try printUsage(args[0]);
        std.process.exit(1);
    }

    const server_path = args[1];
    const server_args = args[2..];

    std.debug.print("\n  LSP Health Checker\n", .{});
    std.debug.print("  Server: {s}\n\n", .{server_path});

    var health_checker = try checker.HealthChecker.init(allocator, server_path, server_args);
    defer health_checker.deinit();

    const results = try health_checker.runAllChecks();
    defer allocator.free(results);

    try display.renderTable(allocator, results);
}

fn printUsage(program: []const u8) !void {
    std.debug.print(
        \\
        \\  lhc - LSP Health Checker
        \\
        \\  Usage: {s} <lsp-server-path> [server-args...]
        \\
        \\  Examples:
        \\    {s} rust-analyzer
        \\    {s} clangd --log=error
        \\    {s} zls
        \\    {s} pyright-langserver --stdio
        \\
        \\  Checks performed:
        \\    · initialize / initialized handshake
        \\    · textDocument/didOpen
        \\    · textDocument/hover
        \\    · textDocument/signatureHelp
        \\    · textDocument/completion
        \\    · textDocument/definition
        \\    · textDocument/references
        \\    · textDocument/documentSymbol
        \\    · textDocument/formatting
        \\    · textDocument/codeAction
        \\    · textDocument/rename
        \\    · textDocument/inlayHint
        \\    · workspace/symbol
        \\    · shutdown / exit
        \\
    , .{ program, program, program, program, program });
}
