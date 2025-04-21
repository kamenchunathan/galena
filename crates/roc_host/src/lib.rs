use roc::roc_main;
use roc_std::{RocBox, RocStr};
use std::io::Write;

mod roc;

#[no_mangle]
pub extern "C" fn rust_main() -> isize {
    let res;
    unsafe {
        res = roc_main(0);
    };
    println!("Hello world");
    dbg!(res);

    0
}
