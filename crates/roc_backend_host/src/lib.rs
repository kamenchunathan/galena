use std::sync::OnceLock;

use tokio::{runtime::Runtime, sync::mpsc::Sender};

mod roc;
mod server;

#[derive(Debug, Clone)]
pub struct MessageInfo {
    pub client_id: String,
    pub msg_bytes: String,
}

pub static ASYNC_RUNTIME: OnceLock<Runtime> = OnceLock::new();
pub static CHANNEL_SENDER: OnceLock<Sender<MessageInfo>> = OnceLock::new();

#[no_mangle]
pub extern "C" fn rust_main() -> isize {
    match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => {
            ASYNC_RUNTIME
                .get_or_init(|| runtime)
                .block_on(async { server::run_server().await });

            0
        }
        Err(err) => {
            eprintln!("Error initializing tokio multithreaded runtime: {}", err);

            1
        }
    }
}
