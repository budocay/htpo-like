use axum::{Router, Server, ServiceExt};
use axum::routing::get;

#[tokio::main]
async fn main() {
    let router = Router::new().route("/", get(root_get));

    let server = Server::bind(&"0.0.0.0:8082".parse().unwrap()).serve(router.into_make_service());
    let addr = server.local_addr();
    println!("Listening on {addr}");
    server.await.unwrap();
}

async fn root_get() -> &'static str {
    "Hi from Axum!"
}