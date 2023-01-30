use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Ip {
    pub ip: String,
}

pub async fn get_public_ip() -> Result<String> {
    let ip = reqwest::get("https://api.ipify.org?format=json")
        .await?
        .json::<Ip>()
        .await?;

    Ok(ip.ip)
}
