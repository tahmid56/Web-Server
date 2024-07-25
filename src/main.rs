use axum::{response::Html, routing::get, Router};
use axum::extract::{Path, State};
use axum::extract::Query;
use std::collections::HashMap;
use std::sync::atomic::{self, AtomicUsize};
use axum::http::HeaderMap;
use std::sync::Arc;

struct MyCounter{
    counter: AtomicUsize,
}

struct MyConfig{
    text: String,
}

#[tokio::main]
async fn main() {
    let shared_counter = Arc::new(MyCounter{
        counter: AtomicUsize::new(0),
    });
    let shared_text = Arc::new(MyConfig{
        text: "This is my configuration".to_string(),
    });
    let app = Router::new()
        .route("/", get(handler))
        .route("/book/:id", get(path_extract))
        .route("/book", get(query_extract))
        .route("/header", get(header_extract))
        .with_state(shared_counter);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
        .await
        .unwrap();

    println!("Listening on http://127.0.0.1:3001");

    axum::serve(listener, app).await.unwrap();
}

async fn handler(
    State(config): State<Arc<MyCounter>>
) -> Html<String> {
    config.counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    Html(format!("<h1>You visited {} times</h1>", config.counter.load(std::sync::atomic::Ordering::Relaxed)))
}

async fn path_extract(Path(id): Path<i32>) -> Html<String> {
    Html(format!("Hello book {}", id))
}

async fn query_extract(Query(params): Query<HashMap<String, String>>) -> Html<String> {
    Html(format!("{params:#?}"))
}

async fn header_extract(headers: HeaderMap) -> Html<String> {
    Html(format!("{headers:#?}"))
}