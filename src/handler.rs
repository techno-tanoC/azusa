mod download;
mod request;
mod response;

use std::sync::Arc;

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
    pub fn build() -> Router {
        let engine = Engine::build();
        let state = Arc::new(State { engine });

        Router::new()
            .route("/download", get(download::index).post(download::start).delete(download::cancel))
            .fallback(not_found)
            .with_state(state)
    }
}

async fn not_found() -> AppError {
    NotFound.into()
}
