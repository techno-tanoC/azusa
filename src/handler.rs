mod download;
mod request;
mod response;

use std::{path::Path, sync::Arc};

use anyhow::Result;
use axum::{routing::get, Router};

use crate::Engine;

use self::response::{AppError, NotFound};

type AppState = Arc<State>;

struct State {
    engine: Engine,
}

pub struct Api;

impl Api {
    #[rustfmt::skip]
    pub fn build(path: impl AsRef<Path>) -> Result<Router> {
        let engine = Engine::build(path)?;
        let state = Arc::new(State { engine });

        let router = Router::new()
            .route("/download", get(download::index).post(download::start).delete(download::cancel))
            .fallback(not_found)
            .with_state(state);
        Ok(router)
    }
}

async fn not_found() -> AppError {
    NotFound.into()
}
