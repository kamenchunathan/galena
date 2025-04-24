const std = @import("std");
const str = @import("glue/str.zig");
const builtin = @import("builtin");

const Allocator = std.mem.Allocator;
const RocStr = str.RocStr;

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
pub export fn js_alloc_bytes(size: usize) ?[*]const u8 {
    const ptr = malloc(size) orelse return null;
    return @as([*]const u8, @ptrCast(@alignCast(ptr)));
}

// For JS to notify the WASM module that it finished writing to memory
pub export fn js_write_res(ptr: [*]const u8, len: usize) void {
    _ = ptr;
    _ = len;
}

const Slice = packed struct { ptr: ?[*]const u8, len: usize };
// Library code
fn pack_pointer_and_length(ptr: [*]const u8, len: usize) u64 {
    const ptr_addr = @intFromPtr(ptr);

    std.debug.assert(ptr_addr <= 0xFFFFFFFF);
    std.debug.assert(len <= 0xFFFFFFFF);

    return ((@as(u64, @intCast(len)) << 32) | @as(u64, @intCast(ptr_addr)));
}

pub export fn js_greet_person(name_ptr: [*]const u8, name_len: usize) Slice {
    // Get a temporary allocator for this operation
    var buffer: [1000]u8 = undefined;
    var fba = std.heap.FixedBufferAllocator.init(&buffer);
    const allocator = fba.allocator();

    var greeting: []u8 = undefined;
    const name_bytes = name_ptr[0..name_len];
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

// Imports

extern fn roc__mainForHost_1_exposed(input: i32) callconv(.C) i32;

pub export fn main() u8 {
    _ = roc__mainForHost_1_exposed(0);

    return 0;
}
