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
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_handler() -> Response {
    Response::builder()
        .status(307)
        .header("Location", "https://github.com/andrewgazelka/hyperion")
        .body(Body::empty())
        .unwrap()
}

async fn post_handler(headers: HeaderMap) -> Response {
    if let Some(key) = headers.get("KEY") {
        if key.as_bytes() == include_str!("../secret").trim().as_bytes() {
            eprintln!("acquiring restart lock");
            let _lock = RESTART_LOCK.lock().await;
            eprintln!("restarting");
            let _ = Command::new("podman").arg("pull").arg("ghcr.io/andrewgazelka/hyperion/tag:latest").status().await;
            let _ = Command::new("podman").arg("pull").arg("ghcr.io/andrewgazelka/hyperion/hyperion-proxy:latest").status().await;
            let _ = Command::new("podman").arg("stop").arg("tag").status().await;
            let _ = Command::new("podman").arg("stop").arg("hyperion-proxy").status().await;
            let _ = Command::new("podman").arg("rm").arg("hyperion-proxy").status().await;
            let _ = Command::new("podman").arg("run").arg("-d").arg("--name").arg("tag").arg("-p").arg("127.0.0.1:35565:35565").arg("tag").status().await;
            let _ = Command::new("podman").arg("run").arg("-d").arg("--name").arg("tag").arg("-p").arg("10.0.0.41:25565:25565").arg("hyperion-proxy").arg("0.0.0.0:25565").status().await;
            eprintln!("finished restart");
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
