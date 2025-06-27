use core::ffi::c_void;
use std::alloc::{GlobalAlloc, Layout};

use roc_std::{RocBox, RocStr};

use crate::ALLOC;

#[derive(Clone, Debug)]
pub struct Model {
    pub inner: RocBox<()>,
}

impl Model {
    // From the basic webserver platform
    pub unsafe fn init(model: RocBox<()>) -> Self {
        // Set the refcount to constant to ensure this never gets freed.
        // This also makes it thread-safe.
        let data_ptr: *mut usize = std::mem::transmute(model);
        let rc_ptr = data_ptr.offset(-1);
        let max_refcount = 0;
        *rc_ptr = max_refcount;

        Self {
            inner: std::mem::transmute::<*mut usize, roc_std::RocBox<()>>(data_ptr),
        }
    }
}

unsafe impl Send for Model {}
unsafe impl Sync for Model {}

#[no_mangle]
pub unsafe extern "C" fn roc_alloc(size: usize, alignment: u32) -> *mut c_void {
    ALLOC.alloc(Layout::from_size_align_unchecked(size, alignment as usize)) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn roc_realloc(
    ptr: *mut c_void,
    new_size: usize,
    old_size: usize,
    alignment: u32,
) -> *mut c_void {
    ALLOC.realloc(
        ptr as *mut u8,
        Layout::from_size_align_unchecked(old_size, alignment as usize),
        new_size,
    ) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn roc_dealloc(ptr: *mut c_void, alignment: u32) {
    ALLOC.dealloc(
        ptr as *mut u8,
        Layout::from_size_align_unchecked(1, alignment as usize),
    );
}

#[no_mangle]
pub unsafe extern "C" fn roc_panic(msg: *mut RocStr, tag_id: u32) {
    // TODO: Use console log
    match tag_id {
        0 => {
            eprintln!("Roc standard library hit a panic: {}", &*msg);
        }
        1 => {
            eprintln!("Application hit a panic: {}", &*msg);
        }
        _ => unreachable!(),
    }

    std::process::exit(1);
}

#[no_mangle]
pub unsafe extern "C" fn roc_dbg(loc: *mut RocStr, msg: *mut RocStr, src: *mut RocStr) {
    // TODO: Use console log
    eprintln!("[{}] {} = {}", &*loc, &*src, &*msg);
}

#[no_mangle]
pub unsafe extern "C" fn roc_memset(dst: *mut c_void, c: i32, n: usize) -> *mut c_void {
    std::ptr::write_bytes(dst as *mut u8, c as u8, n);
    dst
}
