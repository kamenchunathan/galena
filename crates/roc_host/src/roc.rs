use core::ffi::c_void;
use roc_std::{RocBox, RocStr};

#[derive(Clone, Debug)]
pub struct Model {
    model: RocBox<()>,
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
            model: std::mem::transmute::<*mut usize, roc_std::RocBox<()>>(data_ptr),
        }
    }
}

unsafe impl Send for Model {}
unsafe impl Sync for Model {}

pub fn call_roc_backend_init() -> RocBox<()> {
    extern "C" {
        #[link_name = "roc__backend_init_for_host_1_exposed"]
        pub fn caller() -> RocBox<()>;

        // #[link_name = "roc__backend_init_for_host_1_exposed_size"]
        // fn size() -> i64;
    }

    unsafe { caller() }
}

pub fn call_roc_backend_update(mut model: Model, client_id: &str, session_id: &str, msg: &str) {
    extern "C" {
        #[link_name = "roc__backend_update_for_host_1_exposed"]
        pub fn caller(
            model: RocBox<()>,
            client_id: RocStr,
            session_id: RocStr,
            msg_bytes: RocStr,
        ) -> RocBox<()>;

        // #[link_name = "roc__backend_init_for_host_1_exposed_size"]
        // fn size() -> i64;
    }

    let client_id = RocStr::from(client_id);
    let session_id = RocStr::from(session_id);
    let msg = RocStr::from(msg);

    unsafe {
        let updated_roc_model = caller(model.model, client_id, session_id, msg);
        model = Model::init(updated_roc_model);
    };
}

#[no_mangle]
pub unsafe extern "C" fn roc_fx_send_to_backend_impl(_: &RocStr) {
    // This should only be called by the frontend
    eprintln!("Should only be called from frontend");
    std::process::exit(1);
}

#[no_mangle]
pub unsafe extern "C" fn roc_alloc(size: usize, _alignment: u32) -> *mut c_void {
    libc::malloc(size)
}

#[no_mangle]
pub unsafe extern "C" fn roc_realloc(
    c_ptr: *mut c_void,
    new_size: usize,
    _old_size: usize,
    _alignment: u32,
) -> *mut c_void {
    libc::realloc(c_ptr, new_size)
}

#[no_mangle]
pub unsafe extern "C" fn roc_dealloc(c_ptr: *mut c_void, _alignment: u32) {
    libc::free(c_ptr);
}

#[no_mangle]
pub unsafe extern "C" fn roc_panic(msg: *mut RocStr, tag_id: u32) {
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
    eprintln!("[{}] {} = {}", &*loc, &*src, &*msg);
}

#[no_mangle]
pub unsafe extern "C" fn roc_memset(dst: *mut c_void, c: i32, n: usize) -> *mut c_void {
    libc::memset(dst, c, n)
}

#[cfg(unix)]
#[no_mangle]
pub unsafe extern "C" fn roc_getppid() -> libc::pid_t {
    libc::getppid()
}

#[cfg(unix)]
#[no_mangle]
pub unsafe extern "C" fn roc_mmap(
    addr: *mut libc::c_void,
    len: libc::size_t,
    prot: libc::c_int,
    flags: libc::c_int,
    fd: libc::c_int,
    offset: libc::off_t,
) -> *mut libc::c_void {
    libc::mmap(addr, len, prot, flags, fd, offset)
}

#[cfg(unix)]
#[no_mangle]
pub unsafe extern "C" fn roc_shm_open(
    name: *const libc::c_char,
    oflag: libc::c_int,
    mode: libc::mode_t,
) -> libc::c_int {
    libc::shm_open(name, oflag, mode as libc::c_uint)
}
