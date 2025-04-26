use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;

use axum::extract::{Path, Query};
use axum::extract::{Request, State};
use axum::http::HeaderMap;
use axum::middleware::{self, Next};
use axum::response::IntoResponse;
use axum::{response::Html, routing::get, Router};
use axum::{Extension, Json};
use reqwest::StatusCode;
use std::sync::{Arc, Mutex};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::fmt::format::FmtSpan;

pub struct MyCounter {
    counter: AtomicUsize,
}

pub struct MyConfig {
    text: String,
}

struct MyState(i32);

fn service_one() -> Router {
    let mystate = Arc::new(Mutex::new(MyState(0)));
    Router::new()
        .route("/", get(service_one_handler))
        .with_state(mystate)
}

fn service_two() -> Router {
    Router::new().route("/", get(|| async { Html("<h1>Service Two</h1>") }))
}

#[tokio::main]
async fn main() {
    let shared_counter = Arc::new(MyCounter {
        counter: AtomicUsize::new(0),
    });

    let shared_config = Arc::new(MyConfig {
        text: "My Config".to_string(),
    });
    // tracing_subscriber::fmt::init();

    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .with_span_events(FmtSpan::CLOSE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    info!("Starting server...");
    let app = Router::new()
        .nest("/1", service_one())
        .nest("/2", service_two())
        .route("/", get(handler))
        .route("/book/{id}", get(path_extractor))
        .route("/book", get(query_extractor))
        .route("/header", get(header_extractor))
        .route("/inc", get(increment_counter))
        .route("/increment", get(increment_handler))
        .route("/statuscode200", get(status_code_handler))
        .route("/intoresponse", get(into_response_handler))
        .route("/header_authentication", get(header_authentication_handler))
        .layer(Extension(shared_counter))
        .layer(Extension(shared_config))
        // .layer(TraceLayer::new_for_http())
        // .route_layer(middleware::from_fn(auth))
        .fallback_service(ServeDir::new("web"));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    info!("Listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn service_one_handler(
    Extension(mycounter): Extension<Arc<MyCounter>>,
    State(state): State<Arc<Mutex<MyState>>>,
) -> Html<String> {
    mycounter
        .counter
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    let mut state_guard = state.lock().unwrap();
    state_guard.0 += 1;
    let state_value = state_guard.0;
    drop(state_guard);
    Html(format!(
        "<h1>Service One counter: {}</h1> <br/> <h1>State: {}",
        mycounter.counter.load(std::sync::atomic::Ordering::Relaxed),
        state_value
    ))
}

async fn increment_counter(Extension(mycounter): Extension<Arc<MyCounter>>) -> Json<usize> {
    println!("/inc service called");
    mycounter
        .counter
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    Json(mycounter.counter.load(std::sync::atomic::Ordering::Relaxed))
}

async fn increment_handler() -> Html<String> {
    let count = reqwest::get("http://localhost:3000/inc")
        .await
        .unwrap()
        .json::<i32>()
        .await
        .unwrap();
    Html(format!("<h1>Count: {}</h1>", count))
}

async fn handler(
    Extension(mycounter): Extension<Arc<MyCounter>>,
    Extension(myconfig): Extension<Arc<MyConfig>>,
) -> Html<String> {
    tracing::info!("Handler called");
    let counter = mycounter
        .counter
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    Html(format!(
        "<h1>Counter: {}</h1> <br/> <h1> Config: {}",
        counter, myconfig.text
    ))
}

async fn path_extractor(Path(id): Path<String>) -> Html<String> {
    Html(format!("<h1>Book ID: {id}</h1>"))
}

async fn query_extractor(Query(params): Query<HashMap<String, String>>) -> Html<String> {
    Html(format!("<h1>Query Params: {:#?}</h1>", params))
}

async fn header_extractor(headers: HeaderMap) -> Html<String> {
    Html(format!("{headers:#?}"))
}

async fn status_code_handler() -> StatusCode {
    StatusCode::OK
}

async fn into_response_handler() -> Result<impl IntoResponse, (StatusCode, String)> {
    let start = std::time::SystemTime::now();
    let second_wrapped = start
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Bad Clock".to_string()))?
        .as_secs()
        % 3;

    let divided = 100u64
        .checked_div(second_wrapped)
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "div by 0".to_string()))?;

    Ok(Json(divided))
}

async fn header_authentication_handler(headers: HeaderMap) -> impl IntoResponse {
    if let Some(id) = headers.get("x-request-id") {
        Html(format!("<h1>Request ID: {:?}</h1>", id))
    } else {
        Html("x-request-id not found".to_string())
    }
}

async fn auth(
    headers: HeaderMap,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if let Some(id) = headers.get("x-request-id") {
        println!("x-request-id: {:?}", id);
        Ok(next.run(req).await)
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            "x-request-id not found".to_string(),
        ))
    }
}
