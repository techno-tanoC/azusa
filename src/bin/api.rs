use std::sync::Arc;

use anyhow::Result;
use axum::{
    extract::{Path, State},
    routing::{delete, get},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()?;

    let state = new_app_state();
    let app = axum::Router::new()
        .route("/downloads", get(index).post(start))
        .route("/downloads/:id", delete(cancel))
        .with_state(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

type AppState = Arc<azusa::Engine>;

fn new_app_state() -> AppState {
    let engine = azusa::Engine::default();
    Arc::new(engine)
}

#[derive(Debug, Clone, Deserialize)]
struct Params {
    url: String,
    title: String,
    ext: String,
}

#[derive(Debug, Clone, Deserialize)]
struct Id {
    id: Uuid,
}

async fn index(engine: State<AppState>) -> Json<Vec<azusa::Item>> {
    axum::Json(engine.to_items().await)
}

async fn start(engine: State<AppState>, axum::Json(params): Json<Params>) {
    tokio::spawn(async move {
        engine.download(params.url, params.title, params.ext).await;
    });
}

async fn cancel(engine: State<AppState>, id: Path<Id>) {
    engine.cancel(id.id).await;
}
