#[macro_use] extern crate rocket;

mod downloader;

use anyhow::Result;
use rocket_contrib::json::{Json, JsonValue, json};
use serde::Deserialize;

#[get("/")]
async fn index() -> JsonValue {
    json!({
        "status": "ok",
        "downloads": [
            {
                "id": "1",
                "url": "http://example.com",
                "name": "example",
                "total": 1000,
                "size": 100
            },
            {
                "id": "2",
                "url": "http://example.com",
                "name": "example",
                "total": 1000,
                "size": 200
            },
            {
                "id": "3",
                "url": "http://example.com",
                "name": "example",
                "total": 1000,
                "size": 300
            },
            {
                "id": "4",
                "url": "http://example.com",
                "name": "example",
                "total": 1000,
                "size": 900
            },
        ]
    })
}

#[derive(Debug, Clone, Deserialize)]
struct Start {
    url: String,
    name: String,
    ext: String,
}

#[post("/", format = "json", data = "<params>")]
async fn start(params: Json<Start>) -> JsonValue {
    dbg!(params);

    json!({
        "status": "ok"
    })
}

#[derive(Debug, Clone, Deserialize)]
struct Cancel {
    id: String,
}

#[delete("/<id>")]
async fn cancel(id: String) -> JsonValue {
    dbg!(id);

    json!({
        "status": "ok"
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    rocket::ignite()
        .mount("/downloads", routes![index, start, cancel])
        .launch()
        .await?;

    Ok(())
}
