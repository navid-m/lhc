const std = @import("std");
const checker = @import("checker.zig");

// ANSI colour codes
const RESET = "\x1b[0m";
const BOLD = "\x1b[1m";
const DIM = "\x1b[2m";

const GREEN = "\x1b[32m";
const RED = "\x1b[31m";
const YELLOW = "\x1b[33m";
const CYAN = "\x1b[36m";
const BLUE = "\x1b[34m";
const MAGENTA = "\x1b[35m";
const WHITE = "\x1b[97m";

// Unicode box drawing (rounded corners)
const TL = "╭";
const TR = "╮";
const BL = "╰";
const BR = "╯";
const H = "─";
const V = "│";
const TM = "┬";
const BM = "┴";
const LM = "├";
const RM = "┤";
const CR = "┼";

// Status icons
const ICON_PASS = "✓";
const ICON_FAIL = "✗";
const ICON_SKIP = "○";
const ICON_TIME = "◌";

const Outer = struct {
    pub fn writeAll(self: *const Outer, x: []const u8) void {
        _ = self;
        std.debug.print("{s}", .{x});
    }

    pub fn writeByte(self: *const Outer, b: u8) void {
        _ = self;
        std.debug.print("{c}", .{b});
    }

    pub fn print(self: *const Outer, comptime fmt: []const u8, args: anytype) void {
        _ = self;
        std.debug.print(fmt, args);
    }
};

pub fn renderTable(allocator: std.mem.Allocator, results: []const checker.CheckResult) !void {
    _ = allocator;

    // Column widths (fixed, wide enough for all content)
    const W_STATUS = 6;
    const W_NAME = 22;
    const W_METHOD = 34;
    const W_DETAIL = 22;
    const W_LATENCY = 10;

    // Summary counts
    var passed: usize = 0;
    var failed: usize = 0;
    var skipped: usize = 0;
    var timed_out: usize = 0;
    for (results) |r| {
        switch (r.status) {
            .passed => passed += 1,
            .failed => failed += 1,
            .skipped => skipped += 1,
            .timeout => timed_out += 1,
        }
    }

    // ── Top border ────────────────────────────────────────────────────────────
    const out = Outer{};
    out.writeAll("  " ++ TL);
    hline(out, W_STATUS + 2);
    out.writeAll(TM);
    hline(out, W_NAME + 2);
    out.writeAll(TM);
    hline(out, W_METHOD + 2);
    out.writeAll(TM);
    hline(out, W_DETAIL + 2);
    out.writeAll(TM);
    hline(out, W_LATENCY + 2);
    out.writeAll(TR ++ "\n");

    // ── Header row ────────────────────────────────────────────────────────────
    out.writeAll("  " ++ V ++ " ");
    out.writeAll(BOLD ++ WHITE);
    try padRight(out, "Status", W_STATUS);
    out.writeAll(RESET ++ " " ++ V ++ " ");
    out.writeAll(BOLD ++ WHITE);
    try padRight(out, "Check", W_NAME);
    out.writeAll(RESET ++ " " ++ V ++ " ");
    out.writeAll(BOLD ++ WHITE);
    try padRight(out, "LSP Method", W_METHOD);
    out.writeAll(RESET ++ " " ++ V ++ " ");
    out.writeAll(BOLD ++ WHITE);
    try padRight(out, "Detail", W_DETAIL);
    out.writeAll(RESET ++ " " ++ V ++ " ");
    out.writeAll(BOLD ++ WHITE);
    try padRight(out, "Latency", W_LATENCY);
    out.writeAll(RESET ++ " " ++ V ++ "\n");

    // ── Header separator ──────────────────────────────────────────────────────
    out.writeAll("  " ++ LM);
    hline(out, W_STATUS + 2);
    out.writeAll(CR);
    hline(out, W_NAME + 2);
    out.writeAll(CR);
    hline(out, W_METHOD + 2);
    out.writeAll(CR);
    hline(out, W_DETAIL + 2);
    out.writeAll(CR);
    hline(out, W_LATENCY + 2);
    out.writeAll(RM ++ "\n");

    // ── Data rows ─────────────────────────────────────────────────────────────
    for (results, 0..) |r, i| {
        const icon, const color = switch (r.status) {
            .passed => .{ ICON_PASS, GREEN },
            .failed => .{ ICON_FAIL, RED },
            .skipped => .{ ICON_SKIP, YELLOW },
            .timeout => .{ ICON_TIME, MAGENTA },
        };

        // Row separator (between data rows)
        if (i > 0) {
            out.writeAll("  " ++ LM);
            hline(out, W_STATUS + 2);
            out.writeAll(CR);
            hline(out, W_NAME + 2);
            out.writeAll(CR);
            hline(out, W_METHOD + 2);
            out.writeAll(CR);
            hline(out, W_DETAIL + 2);
            out.writeAll(CR);
            hline(out, W_LATENCY + 2);
            out.writeAll(RM ++ "\n");
        }

        // Status cell
        out.writeAll("  " ++ V ++ " ");
        out.writeAll(BOLD ++ color);
        // Icon + space + label
        var status_buf: [32]u8 = undefined;
        const status_label = @tagName(r.status);
        const status_str = try std.fmt.bufPrint(&status_buf, "{s} {s}", .{ icon, status_label });
        try padRight(out, status_str, W_STATUS);
        out.writeAll(RESET);

        // Name cell
        out.writeAll(" " ++ V ++ " ");
        out.writeAll(BOLD);
        try padRight(out, r.name, W_NAME);
        out.writeAll(RESET);

        // Method cell
        out.writeAll(" " ++ V ++ " ");
        out.writeAll(CYAN);
        try padRight(out, r.method, W_METHOD);
        out.writeAll(RESET);

        // Detail cell
        out.writeAll(" " ++ V ++ " ");
        out.writeAll(DIM);
        try padRight(out, r.detail, W_DETAIL);
        out.writeAll(RESET);

        // Latency cell
        out.writeAll(" " ++ V ++ " ");
        if (r.status == .skipped or r.duration_ms == 0) {
            out.writeAll(DIM);
            try padRight(out, "—", W_LATENCY);
            out.writeAll(RESET);
        } else {
            var lat_buf: [32]u8 = undefined;
            const lat_str = try std.fmt.bufPrint(&lat_buf, "{d} ms", .{r.duration_ms});
            const lat_color = if (r.duration_ms < 100) GREEN else if (r.duration_ms < 1000) YELLOW else RED;
            out.writeAll(lat_color);
            try padRight(out, lat_str, W_LATENCY);
            out.writeAll(RESET);
        }

        out.writeAll(" " ++ V ++ "\n");
    }

    // ── Bottom border ─────────────────────────────────────────────────────────
    out.writeAll("  " ++ BL);
    hline(out, W_STATUS + 2);
    out.writeAll(BM);
    hline(out, W_NAME + 2);
    out.writeAll(BM);
    hline(out, W_METHOD + 2);
    out.writeAll(BM);
    hline(out, W_DETAIL + 2);
    out.writeAll(BM);
    hline(out, W_LATENCY + 2);
    out.writeAll(BR ++ "\n\n");

    // ── Summary ───────────────────────────────────────────────────────────────
    out.writeAll("  Summary  ");
    out.writeAll(GREEN ++ BOLD);
    out.print("{s} {d} passed  ", .{ ICON_PASS, passed });
    out.writeAll(RESET);
    if (failed > 0) {
        out.writeAll(RED ++ BOLD);
        out.print("{s} {d} failed  ", .{ ICON_FAIL, failed });
        out.writeAll(RESET);
    }
    if (timed_out > 0) {
        out.writeAll(MAGENTA ++ BOLD);
        out.print("{s} {d} timed out  ", .{ ICON_TIME, timed_out });
        out.writeAll(RESET);
    }
    if (skipped > 0) {
        out.writeAll(YELLOW ++ DIM);
        out.print("{s} {d} skipped  ", .{ ICON_SKIP, skipped });
        out.writeAll(RESET);
    }
    out.writeAll("\n\n");

    // ── Health verdict ────────────────────────────────────────────────────────
    if (failed == 0 and timed_out == 0) {
        out.writeAll("  " ++ GREEN ++ BOLD ++ "● Server is healthy" ++ RESET ++ "\n\n");
    } else {
        out.writeAll("  " ++ RED ++ BOLD ++ "● Server has issues" ++ RESET ++ "\n\n");
    }
}

fn hline(out: anytype, width: usize) void {
    for (0..width) |_| out.writeAll(H);
}

/// Write `s` padded with spaces to exactly `width` visible chars.
/// Handles multi-byte UTF-8 by counting codepoints.
fn padRight(out: anytype, s: []const u8, width: usize) !void {
    out.writeAll(s);
    const vis = visibleLen(s);
    if (vis < width) {
        for (0..width - vis) |_| out.writeByte(' ');
    }
}

fn visibleLen(s: []const u8) usize {
    var count: usize = 0;
    var i: usize = 0;
    while (i < s.len) {
        const byte = s[i];
        if (byte < 0x80) {
            i += 1;
        } else if (byte < 0xE0) {
            i += 2;
        } else if (byte < 0xF0) {
            i += 3;
        } else {
            i += 4;
        }
        count += 1;
    }
    return count;
}
