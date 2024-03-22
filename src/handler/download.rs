use axum::extract::State;
use serde::Deserialize;
use uuid::Uuid;

use super::{
    request::{Json, Query},
    response::{JsonData, Result},
    AppState,
};

#[derive(Debug, Clone, Deserialize)]
pub struct StartParams {
    url: String,
    name: String,
    ext: String,
}

pub async fn index(state: State<AppState>) -> Result<JsonData<Vec<()>>> {
    let items = state.engine.index().await;
    JsonData::ok(items)
}

pub async fn start(
    state: State<AppState>,
    params: Json<StartParams>,
) -> Result<JsonData<serde_json::Value>> {
    state
        .engine
        .start(&params.url, &params.name, &params.ext)
        .await?;
    JsonData::empty()
}

pub async fn cancel(
    state: State<AppState>,
    id: Query<Uuid>,
) -> Result<JsonData<serde_json::Value>> {
    state.engine.cancel(id.hyphenated()).await?;
    JsonData::empty()
}
