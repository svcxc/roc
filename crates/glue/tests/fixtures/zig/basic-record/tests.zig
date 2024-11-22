const std = @import("std");
const roc_app = @import("test_glue/roc_app/roc_app.zig");

pub fn tests() !void {
    const stdout = std.io.getStdOut().writer();

    const record = roc_app.mainForHost();

    try stdout.print("Record field a was: {}\n", .{record.a});
    try stdout.print("Record field b was: {}\n", .{record.b});
}
