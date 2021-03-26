#[macro_use] extern crate rocket;

use anyhow::Result;
use rocket_contrib::json::{Json, JsonValue, json};
use serde::Deserialize;

use rocket::{Request, Response};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::{Header, ContentType, Method, Status};
use std::io::Cursor;

pub struct CORS();

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to requests",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        if request.method() == Method::Options || response.content_type() == Some(ContentType::JSON) {
            response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
            response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, DELETE, OPTIONS"));
            response.set_header(Header::new("Access-Control-Allow-Headers", "Content-Type"));
            response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
        }

        if request.method() == Method::Options {
            response.set_status(Status::Ok);
            response.set_header(ContentType::Plain);
            response.set_sized_body(0, Cursor::new(""));
        }
    }
}

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
        .attach(CORS())
        .mount("/downloads", routes![index, start, cancel])
        .launch()
        .await?;

    Ok(())
}
