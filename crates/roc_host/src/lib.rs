use roc::roc_main;
use roc_std::RocStr;
use std::io::Write;

mod roc;

#[no_mangle]
pub extern "C" fn rust_main() -> isize {
    let mut roc_str = RocStr::default();
    unsafe { roc_main(&mut roc_str) };

    if let Err(e) = std::io::stdout().write_all(roc_str.as_bytes()) {
        panic!("Writing to stdout failed! {:?}", e);
    }

    // roc_str will not print without flushing if it does not contain a newline and you're using --linker=legacy
    if let Err(e) = std::io::stdout().flush() {
        panic!("Failed to flush stdout: {:?}", e);
    }

    0
}
