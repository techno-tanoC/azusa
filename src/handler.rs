mod request;
mod response;

use axum::{routing::get, Router};

use self::response::{AppError, JsonData, NotFound, Result};

pub struct Api;

impl Api {
    #[rustfmt::skip]
    pub fn build() -> Router {
        Router::new()
            .route("/", get(index))
            .fallback(not_found)
    }
}

async fn index() -> Result<JsonData<serde_json::Value>> {
    JsonData::empty()
}

async fn not_found() -> AppError {
    NotFound.into()
}
