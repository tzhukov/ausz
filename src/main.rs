use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(root_handler));

    let addr = "0.0.0.0:3000";

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

async fn root_handler() -> &'static str {
    "Reached the root"
}
