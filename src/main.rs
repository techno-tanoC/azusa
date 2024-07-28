use std::{env, net::SocketAddr, sync::Arc};

use anyhow::Result;
use axum::{
    extract::{Path, State},
    routing::{delete, get},
    Json, Router,
};
use azusa::{Engine, Item};
use serde::Deserialize;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    let port = env::var("PORT").unwrap_or("3000".to_string()).parse()?;
    let volume = env::var("VOLUME").expect("VOLUME is not found");

    let engine = Arc::new(Engine::new(volume));
    let app = build(engine);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn build(engine: Arc<Engine>) -> Router {
    Router::new()
        .route("/downloads", get(index).post(start))
        .route("/downloads/:id", delete(cancel))
        .with_state(engine)
}

async fn index(engine: State<Arc<Engine>>) -> Json<Vec<Item>> {
    let items = engine.index().await;
    Json(items)
}

#[derive(Debug, Clone, Deserialize)]
struct Params {
    url: String,
    title: String,
    ext: String,
}

async fn start(engine: State<Arc<Engine>>, params: Json<Params>) {
    const FILE_THRESHOLD: u64 = 100 * 1024;
    let Params { url, title, ext } = params.0;
    tokio::spawn(async move {
        if let Err(e) = engine.download(url, title, ext, Some(FILE_THRESHOLD)).await {
            println!("{e}");
        }
    });
}

#[derive(Debug, Clone, Deserialize)]
struct Id {
    id: Uuid,
}

async fn cancel(engine: State<Arc<Engine>>, id: Path<Id>) {
    engine.abort(id.id).await;
}
