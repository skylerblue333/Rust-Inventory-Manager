use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use serde::Deserialize;
use std::env;

#[derive(Deserialize)]
struct UpsertRequest {
    sku: String,
    name: String,
    quantity: i64,
}

#[derive(Deserialize)]
struct AdjustRequest {
    delta: i64,
}

fn item_json(item: app::Item) -> serde_json::Value {
    serde_json::json!({
        "sku": item.sku,
        "name": item.name,
        "quantity": item.quantity
    })
}

fn json_response(status: StatusCode, value: serde_json::Value) -> Response {
    (status, Json(value)).into_response()
}

async fn upsert(
    State(inventory): State<app::Inventory>,
    Json(req): Json<UpsertRequest>,
) -> Response {
    match inventory.upsert(&req.sku, &req.name, req.quantity) {
        Ok(item) => json_response(StatusCode::OK, item_json(item)),
        Err(error) => json_response(StatusCode::BAD_REQUEST, serde_json::json!({"error": error})),
    }
}

async fn get_item(
    State(inventory): State<app::Inventory>,
    Path(sku): Path<String>,
) -> Response {
    match inventory.get(&sku) {
        Ok(Some(item)) => json_response(StatusCode::OK, item_json(item)),
        Ok(None) => json_response(
            StatusCode::NOT_FOUND,
            serde_json::json!({"error": "sku not found"}),
        ),
        Err(error) => json_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            serde_json::json!({"error": error}),
        ),
    }
}

async fn list_items(State(inventory): State<app::Inventory>) -> Response {
    match inventory.list() {
        Ok(items) => (
            StatusCode::OK,
            Json(items.into_iter().map(item_json).collect::<Vec<_>>()),
        )
            .into_response(),
        Err(error) => json_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            serde_json::json!({"error": error}),
        ),
    }
}

async fn adjust(
    State(inventory): State<app::Inventory>,
    Path(sku): Path<String>,
    Json(req): Json<AdjustRequest>,
) -> Response {
    match inventory.adjust(&sku, req.delta) {
        Ok(item) => json_response(StatusCode::OK, item_json(item)),
        Err("sku not found") => json_response(
            StatusCode::NOT_FOUND,
            serde_json::json!({"error": "sku not found"}),
        ),
        Err(error) => json_response(StatusCode::BAD_REQUEST, serde_json::json!({"error": error})),
    }
}

async fn health() -> Response {
    json_response(
        StatusCode::OK,
        serde_json::json!({"status": "ok", "service": "sky-inventory"}),
    )
}

async fn ready(State(inventory): State<app::Inventory>) -> Response {
    match inventory.total_units() {
        Ok(units) => json_response(
            StatusCode::OK,
            serde_json::json!({"status": "ready", "units": units}),
        ),
        Err(error) => json_response(
            StatusCode::SERVICE_UNAVAILABLE,
            serde_json::json!({"error": error}),
        ),
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let bind = env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let inventory = app::Inventory::new();
    let router = Router::new()
        .route("/v1/items", put(upsert).get(list_items))
        .route("/v1/items/{sku}", get(get_item))
        .route("/v1/items/{sku}/adjust", post(adjust))
        .route("/healthz", get(health))
        .route("/readyz", get(ready))
        .with_state(inventory);

    let listener = tokio::net::TcpListener::bind(bind).await?;
    axum::serve(listener, router).await
}
