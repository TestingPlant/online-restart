use http::HeaderMap;
use axum::{
    body::Body,
    routing::get,
    Router,
    response::Response
};
use tokio::{sync::Mutex, process::Command};

static RESTART_LOCK: Mutex<()> = Mutex::const_new(());

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(get_handler).post(post_handler));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    restart().await;
    axum::serve(listener, app).await.unwrap();
}

async fn get_handler() -> Response {
    Response::builder()
        .status(307)
        .header("Location", "https://hyperion.rs")
        .body(Body::empty())
        .unwrap()
}

async fn post_handler(headers: HeaderMap) -> Response {
    if let Some(key) = headers.get("KEY") {
        if key.as_bytes() == include_str!("../secret").trim().as_bytes() {
            tokio::spawn(restart());
            return Response::builder()
                .status(200)
                .body(Body::empty())
                .unwrap();
        }
    }

    Response::builder()
        .status(401)
        .body(Body::empty())
        .unwrap()
}

async fn restart() {
    eprintln!("acquiring restart lock");
    let _lock = RESTART_LOCK.lock().await;
    eprintln!("restarting");
    match Command::new("./restart").status().await {
        Err(e) => eprintln!("RESTART FAILED: {e}"),
        Ok(status) => match status.code() {
            Some(0) => eprintln!("restart success"),
            Some(e) => eprintln!("RESTART FAILED: exited with error code {e}"),
            None => eprintln!("RESTART FAILED: terminated by signal")
        }
    }
}
