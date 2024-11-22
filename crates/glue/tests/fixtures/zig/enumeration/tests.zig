const std = @import("std");
const roc_app = @import("test_glue/roc_app/roc_app.zig");

pub fn tests() !void {
    const stdout = std.io.getStdOut().writer();

    const fooFromRoc = roc_app.mainForHost();
    const fooFromZig: roc_app.MyEnum = .Foo;

    try stdout.print("Foo from Roc was: {s}\n", .{@tagName(fooFromRoc)});
    try stdout.print("Foo from Zig was: {s}\n", .{@tagName(fooFromZig)});

    const fields = @typeInfo(roc_app.MyEnum).Enum.fields;
    const last = fields.len - 1;

    try stdout.writeAll("tags: ");
    inline for (fields, 0..) |field, i| {
        try stdout.writeAll(field.name);
        if (i != last) try stdout.writeAll(", ");
    }
    try stdout.writeAll("\n");
}
