use std::{env, net::SocketAddr};

use anyhow::Result;
use azusa::Api;

#[tokio::main]
async fn main() -> Result<()> {
    let port = env::var("PORT").unwrap_or("3000".to_string()).parse()?;
    let volume = env::var("VOLUME").unwrap_or(".".to_string());

    let app = Api::build(&volume)?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
