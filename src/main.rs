use axum:: {
    routing::{get, post},
    Router
};
use serde::{Serialize, Deserialize};




#[derive(Serialize)]
struct SuccessResponse<T> {
    success: bool,
    data: T,
}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    error: String,
}




#[tokio::main]

async fn main() {
    let app = Router::new().route("/check-health", get(check_health));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn check_health() -> String {
    "The server is working at 3000".to_string()
}