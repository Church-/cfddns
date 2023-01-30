use anyhow::Result;
use reqwest::header;
use serde::{Deserialize, Serialize};

pub struct Cloudflare {
    client: reqwest::Client,
    zone_id: String,
    endpoint: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Records {
    pub result: Vec<Record>,
    pub success: bool,
    result_info: QueryInfo,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Record {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub r#type: String,
    pub name: String,
    pub content: String,
    pub ttl: i16,
    pub proxied: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QueryInfo {
    pub page: i16,
    pub per_page: i16,
    pub count: i16,
    pub total_count: i16,
}

impl Cloudflare {
    pub fn init(token: String, zone_id: String) -> Result<Cloudflare> {
        let mut headers = header::HeaderMap::new();
        let header = format!("Bearer {}", token);
        let auth_header = header::HeaderValue::from_str(&header)?;
        headers.insert("Authorization", auth_header);

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Cloudflare {
            client,
            zone_id,
            endpoint: "https://api.cloudflare.com/client/v4/".to_string(),
        })
    }

    pub async fn get_records(&self) -> Result<Records> {
        let endpoint = format!("{}/zones/{}/dns_records", self.endpoint, self.zone_id);
        let records = self
            .client
            .get(endpoint)
            .send()
            .await?
            .json::<Records>()
            .await?;

        Ok(records)
    }

    pub async fn update_record(&self, mut record: Record) -> Result<()> {
        let endpoint = format!("{}/zones/{}/dns_records", self.endpoint, self.zone_id);
        let record_id = record
            .id
            .ok_or_else(|| anyhow::anyhow!("Unable to get id of DNS Record"))?;
        let record_endpoint = format!("{}/{}", &endpoint, record_id);

        record.id = None;

        self.client
            .put(record_endpoint)
            .json(&record)
            .send()
            .await?;

        Ok(())
    }

    pub async fn create_record(&self, record_name: &str, ip_address: &str) -> Result<()> {
        let endpoint = format!("{}/zones/{}/dns_records", self.endpoint, self.zone_id);
        let record = Record {
            id: None,
            r#type: String::from("A"),
            name: record_name.to_string(),
            content: ip_address.to_string(),
            ttl: 1,
            proxied: false,
        };

        self.client.post(endpoint).json(&record).send().await?;

        Ok(())
    }
}
