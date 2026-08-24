use actix_web::{web, App, HttpResponse, HttpServer, Responder};
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

async fn upsert(
    inventory: web::Data<app::Inventory>,
    req: web::Json<UpsertRequest>,
) -> impl Responder {
    match inventory.upsert(&req.sku, &req.name, req.quantity) {
        Ok(item) => HttpResponse::Ok().json(item_json(item)),
        Err(error) => HttpResponse::BadRequest().json(serde_json::json!({"error": error})),
    }
}

async fn get_item(inventory: web::Data<app::Inventory>, sku: web::Path<String>) -> impl Responder {
    match inventory.get(&sku.into_inner()) {
        Ok(Some(item)) => HttpResponse::Ok().json(item_json(item)),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({"error": "sku not found"})),
        Err(error) => HttpResponse::InternalServerError().json(serde_json::json!({"error": error})),
    }
}

async fn list_items(inventory: web::Data<app::Inventory>) -> impl Responder {
    match inventory.list() {
        Ok(items) => HttpResponse::Ok().json(
            items
                .into_iter()
                .map(item_json)
                .collect::<Vec<serde_json::Value>>(),
        ),
        Err(error) => HttpResponse::InternalServerError().json(serde_json::json!({"error": error})),
    }
}

async fn adjust(
    inventory: web::Data<app::Inventory>,
    sku: web::Path<String>,
    req: web::Json<AdjustRequest>,
) -> impl Responder {
    match inventory.adjust(&sku.into_inner(), req.delta) {
        Ok(item) => HttpResponse::Ok().json(item_json(item)),
        Err("sku not found") => {
            HttpResponse::NotFound().json(serde_json::json!({"error": "sku not found"}))
        }
        Err(error) => HttpResponse::BadRequest().json(serde_json::json!({"error": error})),
    }
}

async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"status": "ok", "service": "sky-inventory"}))
}

async fn ready(inventory: web::Data<app::Inventory>) -> impl Responder {
    match inventory.total_units() {
        Ok(units) => {
            HttpResponse::Ok().json(serde_json::json!({"status": "ready", "units": units}))
        }
        Err(error) => HttpResponse::ServiceUnavailable().json(serde_json::json!({"error": error})),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let bind = env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let inventory = app::Inventory::new();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(inventory.clone()))
            .route("/v1/items", web::put().to(upsert))
            .route("/v1/items", web::get().to(list_items))
            .route("/v1/items/{sku}", web::get().to(get_item))
            .route("/v1/items/{sku}/adjust", web::post().to(adjust))
            .route("/healthz", web::get().to(health))
            .route("/readyz", web::get().to(ready))
    })
    .bind(bind)?
    .run()
    .await
}
