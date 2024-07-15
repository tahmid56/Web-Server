use std::fmt::format;

use axum::{response::Html, routing::get, Router};
use axum::extract::Path;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(handler))
        .route("/book/:id", get(path_extract));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
        .await
        .unwrap();

    println!("Listening on http://127.0.0.1:3001");

    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, world!</h1>")
}

async fn path_extract(Path(id): Path<i32>) -> Html<String> {
    Html(format!("Hello book {}", id))
}
