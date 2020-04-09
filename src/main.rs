#[macro_use] extern crate log;
mod progress;
mod item;
mod lock_copy;
mod download;
mod table;
mod app;
mod error;

use serde::Deserialize;
use std::convert::Infallible;
use warp::Filter;
use warp::http::StatusCode;

use app::App;

#[tokio::main]
async fn main() {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();
    let path = std::env::var("VOLUME").unwrap_or_else(|_| ".".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("Invalit PORT");
    let app = App::new(&path);
    info!("start server on {}", port);
    warp::serve(routes(app)).run(([0, 0, 0, 0], port)).await;

}

async fn fetch(app: App) -> Result<impl warp::Reply, Infallible> {
    debug!("[GET] /download");

    let vec = app.table.to_items().await;
    Ok(warp::reply::json(&vec))
}

#[derive(Deserialize, Debug)]
struct Start {
    url: String,
    name: String,
    ext: String,
}

async fn start(start: Start, app: App) -> Result<impl warp::Reply, Infallible> {
    info!("[POST] /download {:?}", &start);

    tokio::spawn(async move {
        let result = app.download(&start.url, &start.name, &start.ext).await;
        if let Err(e) = result {
            warn!("{:?}", e);
        }
    });

    Ok(StatusCode::CREATED)
}

#[derive(Deserialize, Debug)]
struct Cancel {
    id: String,
}

async fn cancel(cancel: Cancel, app: App) -> Result<impl warp::Reply, Infallible> {
    info!("[DELETE] /download {:?}", &cancel);

    tokio::spawn(async move {
        app.table.cancel(&cancel.id).await;
    });

    Ok(StatusCode::NO_CONTENT)
}

#[allow(clippy::redundant_clone)]
fn routes(app: App) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let get = warp::path!("download")
        .and(warp::get())
        .and(with_app(app.clone()))
        .and_then(fetch);

    let post = warp::path!("download")
        .and(warp::post())
        .and(warp::body::content_length_limit(64 * 1024))
        .and(warp::body::json())
        .and(with_app(app.clone()))
        .and_then(start);

    let delete = warp::path!("download")
        .and(warp::delete())
        .and(warp::query::<Cancel>())
        .and(with_app(app.clone()))
        .and_then(cancel);

    let assets = warp::path("assets")
        .and(warp::fs::dir("assets"));

    let cors = warp::cors().allow_methods(vec!["GET", "POST", "DELETE"]);
    get.or(post).or(delete).or(assets).with(cors)
}

fn with_app(app: App) ->  impl Filter<Extract = (App,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || app.clone())
}
