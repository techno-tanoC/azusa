use std::path::PathBuf;

use anyhow::Result;
use tokio::fs;

pub async fn trusted_root_certificates(
    cert_path: impl Into<PathBuf>,
) -> Result<Vec<reqwest::Certificate>> {
    let path = cert_path.into();
    let mut certs = vec![];
    let mut dir = fs::read_dir(path).await?;
    while let Some(entry) = dir.next_entry().await? {
        let pem = fs::read_to_string(entry.path()).await?;
        let cert = reqwest::tls::Certificate::from_pem(pem.as_bytes())?;
        certs.push(cert);
    }
    Ok(certs)
}
