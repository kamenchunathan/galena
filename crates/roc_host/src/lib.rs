use roc::roc_main;

mod roc;

#[no_mangle]
pub extern "C" fn rust_main() -> isize {
    // unsafe {
    //     roc_main(0);
    // };
    //
    0
}
