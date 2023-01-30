use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub api_token: String,
    pub backtrace: Option<BacktraceConfig>,
    pub domains: Vec<String>,
    pub interval: Option<i64>,
    pub zone_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BacktraceConfig {
    pub token: String,
    pub url: String,
}
