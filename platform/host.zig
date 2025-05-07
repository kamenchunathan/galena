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
// extern fn js_send_to_backend(slice: Slice) void;

// JS DOM manipulation functions
// extern fn js_create_text_node(text: Slice) Slice;
// extern fn js_create_div(
//     attrs: Slice,
//     childIds: Slice,
//     callback: ?*const fn (Slice, Slice, Slice) void,
// ) Slice;
// extern fn js_create_input(
//     attrs: Slice,
//     callback: ?*const fn (Slice, Slice, Slice) void,
// ) Slice;
// extern fn js_render_view(rootId: Slice, targetId: Slice) bool;

// Event handling callback function
// export fn handle_dom_event(eventType: Slice, elementId: Slice, value: Slice) void {
//     const eventTypeStr = RocStr.fromSlice(eventType.to_zig_slice() orelse return);
//     const elementIdStr = RocStr.fromSlice(elementId.to_zig_slice() orelse return);
//     const valueStr = RocStr.fromSlice(value.to_zig_slice() orelse return);
//
//     // Call Roc function to handle the event
//     roc_handle_dom_event(eventTypeStr, elementIdStr, valueStr);
// }

const ViewResult = extern struct { model: *RocBox, view: RocList };

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
extern fn roc__render_view_1_exposed(view: *const RocStr) RocStr;
extern fn roc__handle_dom_event_1_exposed(
    model_ptr: *const RocBox,
    eventType: *const RocStr,
    elementId: *const RocStr,
    value: *const RocStr,
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
    return .{ .ptr = res.view.bytes, .len = res.view.len() };
}

// New function to render the view to the DOM
// pub export fn render_view(target_id: Slice) bool {
//     const res = roc__frontend_host_view_1_exposed(model);
//     model = res.model;

// const rendered_view = roc__render_view_1_exposed(&res.view);
// const rendered_slice = rendered_view.asSlice();
//
// return js_render_view(Slice.from_zig_slice(rendered_slice), target_id);
// }

// Handle DOM events from JavaScript
// fn roc_handle_dom_event(eventType: RocStr, elementId: RocStr, value: RocStr) void {
//     model = roc__handle_dom_event_1_exposed(model, &eventType, &elementId, &value);
//
//     // After handling the event, we may want to update the view
//     // This could be optional depending on your application design
//     _ = render_view(Slice.from_zig_slice("app".ptr[0..3]));
// }

pub export fn handle_ws_message(ws_msg: Slice) void {
    const msg_str = RocStr.fromSlice(Slice.to_zig_slice(ws_msg) orelse return);
    model = roc__frontend_receive_ws_message_for_host_1_exposed(model, &msg_str);
}

pub export fn main() u8 {
    return 0;
}

// Effects
export fn roc_fx_send_to_backend_impl(msg_bytes: *RocStr) callconv(.C) void {
    _ = msg_bytes;
    // js_send_to_backend(Slice.from_zig_slice(msg_bytes.asSlice()));
}

// View rendering effects
// export fn roc_fx_create_text_node_impl(text: *RocStr) callconv(.C) RocStr {
//     const text_slice = Slice.from_zig_slice(text.asSlice());
//     const result_slice = js_create_text_node(text_slice);
//     return RocStr.fromSlice(result_slice.to_zig_slice() orelse "".ptr[0..0]);
// }
//
// export fn roc_fx_create_div_impl(attrs: *RocStr, childIds: *RocStr) callconv(.C) RocStr {
//     const attrs_slice = Slice.from_zig_slice(attrs.asSlice());
//     const childIds_slice = Slice.from_zig_slice(childIds.asSlice());
//     const result_slice = js_create_div(attrs_slice, childIds_slice, handle_dom_event);
//     return RocStr.fromSlice(result_slice.to_zig_slice() orelse "".ptr[0..0]);
// }
//
// export fn roc_fx_create_input_impl(attrs: *RocStr) callconv(.C) RocStr {
//     const attrs_slice = Slice.from_zig_slice(attrs.asSlice());
//     const result_slice = js_create_input(attrs_slice, handle_dom_event);
//     return RocStr.fromSlice(result_slice.to_zig_slice() orelse "".ptr[0..0]);
// }
