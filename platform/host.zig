const builtin = @import("builtin");
const std = @import("std");
const str = @import("glue/str.zig");
const list = @import("glue/list.zig");
const result = @import("glue/result.zig");

const Allocator = std.mem.Allocator;
const FrontendModel = opaque {};
const RocStr = str.RocStr;
const RocList = list.RocList;
const RocResult = result.RocResult;

const Slice = packed struct {
    ptr: ?[*]const u8,
    len: usize,

    pub fn to_zig_slice(self: Slice) ?[]const u8 {
        if (self.ptr) |ptr| {
            return ptr[0..self.len];
        } else {
            return null;
        }
    }

    pub fn from_zig_slice(slice: []const u8) Slice {
        return .{ .ptr = slice.ptr, .len = slice.len };
    }
};

comptime {
    if (builtin.target.cpu.arch != .wasm32) {
        @compileError("This platform is for WebAssembly only. You need to pass `--target wasm32` to the Roc compiler.");
    }
}

const Align = 2 * @alignOf(usize);
extern fn malloc(size: usize) callconv(.C) ?*align(Align) anyopaque;
extern fn realloc(c_ptr: [*]align(Align) u8, size: usize) callconv(.C) ?*anyopaque;
extern fn free(c_ptr: [*]align(Align) u8) callconv(.C) void;
extern fn memcpy(dest: *anyopaque, src: *anyopaque, count: usize) *anyopaque;

export fn roc_alloc(size: usize, alignment: u32) callconv(.C) ?*anyopaque {
    _ = alignment;

    return malloc(size);
}

export fn roc_realloc(c_ptr: *anyopaque, new_size: usize, old_size: usize, alignment: u32) callconv(.C) ?*anyopaque {
    _ = old_size;
    _ = alignment;

    return realloc(@as([*]align(Align) u8, @alignCast(@ptrCast(c_ptr))), new_size);
}

export fn roc_dealloc(c_ptr: *anyopaque, alignment: u32) callconv(.C) void {
    _ = alignment;

    free(@as([*]align(Align) u8, @alignCast(@ptrCast(c_ptr))));
}

export fn roc_dbg(loc: *RocStr, msg: *RocStr, src: *RocStr) void {
    std.debug.print("[{s}] {s} = {s}", .{ loc.asSlice(), src.asSlice(), msg.asSlice() });
}

export fn roc_panic(msg: *RocStr, tag_id: u32) void {
    switch (tag_id) {
        0 => std.debug.print("Roc standard library hit a panic: {s}", .{msg.asSlice()}),
        1 => std.debug.print("Application hit a panic: {s}", .{msg.asSlice()}),
        else => {},
    }

    std.process.exit(0);
}

// Exports
pub export fn js_alloc(size: usize) ?[*]const u8 {
    const ptr = malloc(size) orelse return null;
    return @as([*]const u8, @ptrCast(@alignCast(ptr)));
}

// Library code
pub export fn js_greet_person(name: Slice) Slice {
    // Get a temporary allocator for this operation
    var buffer: [1000]u8 = undefined;
    var fba = std.heap.FixedBufferAllocator.init(&buffer);
    const allocator = fba.allocator();

    var greeting: []u8 = undefined;
    const name_bytes = name.to_zig_slice().?;
    if (std.unicode.utf8ValidateSlice(name_bytes)) {
        greeting = std.fmt.allocPrint(allocator, "Hello, {s}! Welcome to our WebAssembly module.", .{name_bytes}) catch {
            return .{ .ptr = null, .len = 0 };
        };
    } else {
        return .{ .ptr = null, .len = 0 };
    }

    const result_ptr =
        @as(
        [*]u8,
        @ptrCast(@alignCast(
            malloc(greeting.len) orelse return .{
                .ptr = null,
                .len = 0,
            },
        )),
    );
    _ = memcpy(result_ptr, greeting.ptr, greeting.len);

    return .{ .ptr = result_ptr, .len = greeting.len };
}

// Js functions
extern fn sendToBackend(Slice) void;

const ViewResult = extern struct { model: *FrontendModel, view: RocList };

const UpdateResult = extern struct {
    model: *FrontendModel,
    to_backend: RocResult(*FrontendModel, struct {}),
};

var model: *FrontendModel = undefined;

extern fn roc__frontend_init_for_host_1_exposed(input: i32) callconv(.C) *FrontendModel;
extern fn roc__frontend_update_for_host_1_exposed(
    model_ptr: *const FrontendModel,
) UpdateResult;
extern fn roc__frontend_update_for_host_1_exposed_size() u64;
extern fn roc__frontend_view_for_host_1_exposed(model_ptr: *FrontendModel) ViewResult;
extern fn roc__frontend_handle_ws_event_for_host_1_exposed(
    model_ptr: *const FrontendModel,
    msg: *const RocStr,
) UpdateResult;

pub export fn init() void {
    model = roc__frontend_init_for_host_1_exposed(0);
}

pub export fn view() Slice {
    const res = roc__frontend_view_for_host_1_exposed(model);
    model = res.model;
    return .{ .ptr = res.view.bytes, .len = res.view.len() };
}

// Handle DOM events from JavaScript
pub export fn handle_dom_event(callback_id: u32, value: Slice) void {
    _ = callback_id;
    std.debug.print("{?s}", .{ .v = value.to_zig_slice() });

    // const value_str = RocStr.fromSlice(value.to_zig_slice() orelse return);
    const ret = roc__frontend_update_for_host_1_exposed(model);
    model = ret.model;
}

pub export fn handle_ws_message(ws_msg: Slice) void {
    const msg_str = RocStr.fromSlice(Slice.to_zig_slice(ws_msg) orelse return);
    const ret = roc__frontend_handle_ws_event_for_host_1_exposed(model, &msg_str);
    model = ret.model;
}

pub export fn main() u8 {
    return 0;
}

// Effects
export fn roc_fx_send_to_backend_impl(msg_bytes: *RocStr) callconv(.C) void {
    sendToBackend(Slice.from_zig_slice(msg_bytes.asSlice()));
}
