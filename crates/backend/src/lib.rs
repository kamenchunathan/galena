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
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tower_cookies::{CookieManagerLayer, Cookies};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing::{info, instrument};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Debug, Clone)]
struct AppState {
    clients: Arc<Mutex<HashMap<String, SplitSink<WebSocket, Message>>>>,
}

pub async fn run() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let asset_server = ServeDir::new("crates/roc_host/static/");

    let router = Router::new()
        .route(
            "/",
            get_service(ServeFile::new_with_mime(
                "crates/roc_host/static/index.html",
                &mime::TEXT_HTML,
            )),
        )
        .route("/ws", any(ws_handler))
        .nest_service("/static", asset_server)
        .layer(CookieManagerLayer::new())
        .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default()))
        .with_state(AppState {
            clients: Arc::new(Mutex::new(HashMap::new())),
        });

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Unable to bind to port");
    tracing::info!("listening on {}", listener.local_addr().unwrap());

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
    info!("state {:?}", state);
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

    ws.on_upgrade(move |socket| handle_websocket_connection(state, socket, session_id, client_id))
}

async fn handle_websocket_connection(
    AppState { clients }: AppState,
    ws: WebSocket,
    session_id: String,
    client_id: String,
) {
    let (sink, mut stream) = ws.split();
    {
        let clients = Arc::clone(&clients);
        let mut clients = clients.lock().await;
        clients.insert(client_id.clone(), sink);
    }

    // Recieve messages
    while let Some(Ok(Message::Text(msg))) = stream.next().await {
        let clients = Arc::clone(&clients);
        let session_id = session_id.clone();
        let client_id = client_id.clone();

        tokio::spawn(async move {
            if let Some(msg) = call_roc_update_from_frontend(session_id, client_id.clone(), msg) {
                let mut clients = clients.lock().await;
                if let Some(stream) = clients.get_mut(&client_id) {
                    stream
                        .send(msg)
                        .await
                        .expect("unable to send message to websocket");
                }
            }
        });
    }
}

fn call_roc_update_from_frontend(
    _sessiond_id: String,
    _client_id: String,
    _msg: String,
) -> Option<Message> {
    None
}
