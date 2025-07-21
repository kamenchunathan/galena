use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, RwLock};

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::{any, get_service};
use axum::Router;
use base64::alphabet::STANDARD;
use base64::engine::{GeneralPurpose, GeneralPurposeConfig};
use base64::Engine;
use cookie::Cookie;
use futures::stream::SplitSink;
use futures::{SinkExt, StreamExt};
use mime;
use rand::{self, RngCore};
use roc_std::RocStr;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, Mutex};
use tower_cookies::{CookieManagerLayer, Cookies};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing::{debug, error, info, instrument, warn};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::roc::{self, call_roc_backend_init, BackendUpdateReturn, Model, ToFrontend};
use crate::{MessageInfo, CHANNEL_SENDER};

#[derive(Debug, Clone)]
struct AppState {
    clients: Arc<Mutex<HashMap<String, SplitSink<WebSocket, Message>>>>,
    roc_model: Arc<RwLock<Model>>,
}

pub async fn run_server() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let dist_dir = env::var("DIST_DIR").expect("DIST_DIR is not defined");

    debug!("Initializing roc model");
    let roc_model = unsafe {
        let boxed_model = call_roc_backend_init();
        roc::Model::init(boxed_model)
    };

    let clients: Arc<Mutex<HashMap<String, SplitSink<WebSocket, Message>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    debug!("Initializing sender channel");
    let (tx, mut rx) = mpsc::channel(20);
    CHANNEL_SENDER.set(tx).expect("Unable to set sender");

    {
        let clients = Arc::clone(&clients);
        tokio::spawn(async move {
            while let Some(MessageInfo {
                client_id,
                msg_bytes,
            }) = rx.recv().await
            {
                debug!(?client_id, "Receive channel message");
                let mut clients = clients.lock().await;
                match clients.get_mut(&client_id) {
                    Some(sink) => {
                        debug!(?sink, "Sink");
                        sink.send(Message::Text(msg_bytes))
                            .await
                            .unwrap_or_else(|_| {
                                error!("Could not send message through websocket");
                            });
                    }
                    _ => {
                        error!(
                            clients = ?clients.iter(),
                            "Client id not in connected clients"
                        );
                    }
                }
            }
        });
    }

    let router = Router::new()
        .route(
            "/",
            get_service(ServeFile::new_with_mime(
                Path::new(&dist_dir).join("index.html"),
                &mime::TEXT_HTML,
            )),
        )
        .route("/ws", any(ws_handler))
        .fallback_service(ServeDir::new(dist_dir))
        .layer(CookieManagerLayer::new())
        .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default()))
        .with_state(AppState {
            clients,
            roc_model: Arc::new(RwLock::new(roc_model)),
        });

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Unable to bind to port");
    info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Error when starting server");
}

#[instrument]
async fn ws_handler(
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
    cookies: Cookies,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    info!("Websocket connection requested");
    // On reconnect check for a session id or create one
    let session_id = match cookies.get("sessionid") {
        Some(sid) => sid.value().to_string(),
        _ => {
            let mut sid_buf = [0u8; 16];

            // TODO: Move the random number generator and base64 engine to a shared location for
            // each request
            let mut rng = rand::thread_rng();
            rng.fill_bytes(&mut sid_buf);
            let engine = GeneralPurpose::new(&STANDARD, GeneralPurposeConfig::default());
            let sesssion_id = engine.encode(sid_buf);
            cookies.add(Cookie::new("sessionid", sesssion_id.clone()));

            sesssion_id
        }
    };

    let client_id =
        GeneralPurpose::new(&STANDARD, GeneralPurposeConfig::default()).encode(&addr.to_string());

    ws.on_upgrade(move |socket| handle_websocket_connection(state, socket, client_id, session_id))
}

#[instrument(skip(roc_model, ws))]
async fn handle_websocket_connection(
    AppState { clients, roc_model }: AppState,
    ws: WebSocket,
    client_id: String,
    session_id: String,
) {
    let (sink, mut stream) = ws.split();
    {
        let clients = Arc::clone(&clients);
        let mut clients = clients.lock().await;
        clients.insert(client_id.clone(), sink);
    }

    // Recieve messages
    loop {
        match dbg!(stream.next().await) {
            Some(Ok(Message::Text(msg))) => {
                let session_id = session_id.clone();
                let client_id = client_id.clone();
                let roc_model = Arc::clone(&roc_model);
                tokio::spawn(async move {
                    handle_ws_message(roc_model, &client_id, &session_id, &msg).await;
                });
            }

            // None => {
            //     let mut clients = clients.lock().await;
            //     clients.remove(&client_id);
            //     break;
            // }
            e => error!(?e, "Unhandled message"),
        }
    }
}

#[instrument(skip(model))]
async fn handle_ws_message(
    model: Arc<RwLock<Model>>,
    client_id: &str,
    session_id: &str,
    msg: &str,
) {
    debug!("Received message");
    let roc_model = {
        let read_lock = model.read().expect("Could not acquire lock");
        read_lock.clone()
    };

    let BackendUpdateReturn {
        model: updated_model,
        to_frontend,
    } = roc::backend_update_for_host(
        roc_model.clone(),
        RocStr::from(client_id),
        RocStr::from(session_id),
        RocStr::from(msg),
    );

    match model.write() {
        Ok(mut write_lock) => {
            *write_lock = unsafe { Model::init(updated_model) };
        }
        _ => {
            drop(updated_model);
        }
    }

    let to_frontend: Result<_, _> = to_frontend.into();
    let to_frontend = to_frontend.map(|to_frontend: ToFrontend| {
        (
            to_frontend.client_id.as_str().to_owned(),
            to_frontend.message.as_str().to_owned(),
        )
    });
    warn!(?to_frontend, "To Frontend");

    if let Ok((client_id, msg_bytes)) = to_frontend {
        if let Some(tx) = CHANNEL_SENDER.get() {
            tx.send(MessageInfo {
                client_id,
                msg_bytes,
            })
            .await
            .expect("Could not send message");
        }
    }
}
