mod roc;
mod server;

#[no_mangle]
pub extern "C" fn rust_main() -> isize {
    match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => {
            runtime.block_on(async { server::run_server().await });

            0
        }
        Err(err) => {
            eprintln!("Error initializing tokio multithreaded runtime: {}", err);

            1
        }
    }
}
