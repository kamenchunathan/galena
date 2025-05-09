const builtin = @import("builtin");
const std = @import("std");
const str = @import("glue/str.zig");
const list = @import("glue/list.zig");
const result = @import("glue/result.zig");

const Allocator = std.mem.Allocator;
const RocBox = opaque {};
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

// Exports
pub export fn js_alloc(size: usize) ?[*]const u8 {
    const ptr = malloc(size) orelse return null;
    return @as([*]const u8, @ptrCast(@alignCast(ptr)));
}

// For JS to notify the WASM module that it finished writing to memory
pub export fn js_write_res(ptr: [*]const u8, len: usize) void {
    _ = ptr;
    _ = len;
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
        @as([*]u8, @ptrCast(@alignCast(malloc(greeting.len) orelse return .{ .ptr = null, .len = 0 })));
    _ = memcpy(result_ptr, greeting.ptr, greeting.len);

    return .{ .ptr = result_ptr, .len = greeting.len };
}

// Js env functions
extern fn js_send_to_backend(slice: Slice) void;

const ViewResult = extern struct { model: *RocBox, view: RocStr };

var model: *RocBox = undefined;
extern fn roc__frontend_host_init_1_exposed(input: i32) callconv(.C) *RocBox;
extern fn roc__frontend_host_update_1_exposed(
    model_ptr: *const RocBox,
    msg_bytes: *const RocStr,
) *RocBox;
extern fn roc__frontend_host_update_1_exposed_size() u64;
extern fn roc__frontend_host_view_1_exposed(model_ptr: *RocBox) ViewResult;
extern fn roc__frontend_receive_ws_message_for_host_1_exposed(
    model_ptr: *const RocBox,
    msg: *const RocStr,
) *RocBox;

pub export fn init() void {
    model = roc__frontend_host_init_1_exposed(0);
}

pub export fn update(msg_bytes: Slice) void {
    const msg: RocStr = RocStr.fromSlice(msg_bytes.to_zig_slice().?);

    model = roc__frontend_host_update_1_exposed(model, &msg);
}

pub export fn view() Slice {
    const res = roc__frontend_host_view_1_exposed(model);
    model = res.model;
    return Slice.from_zig_slice(res.view.asSlice());
}

pub export fn handle_ws_message(ws_msg: Slice) void {
    const msg_str = RocStr.fromSlice(Slice.to_zig_slice(ws_msg) orelse return);
    model = roc__frontend_receive_ws_message_for_host_1_exposed(model, &msg_str);
}

pub export fn main() u8 {
    return 0;
}

// Effects
export fn roc_fx_send_to_backend_impl(msg_bytes: *RocStr) callconv(.C) void {
    js_send_to_backend(Slice.from_zig_slice(msg_bytes.asSlice()));
}
