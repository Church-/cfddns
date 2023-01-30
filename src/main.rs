use std::{collections::HashMap, fs::File, io::BufReader, time::Duration};

use anyhow::{anyhow, Context, Result};
use backtraceio::ResultExt;
use clap::Parser;
use gethostname::gethostname;
use tokio::time::sleep;
use tracing::info;

use cloudflare::Cloudflare;
use cloudflare::Record;

pub mod args;
pub mod cloudflare;
pub mod config;
pub mod ip;

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    tracing_subscriber::fmt::init();

    info!("Starting Cloudflare DynDNS daemon");

    let args = args::Args::parse();
    let file = File::open(args.config).context("Failed to open config file, file handler")?;
    let reader = BufReader::new(file);
    let config: config::Config =
        serde_json::from_reader(reader).context("Failed to parse config file")?;

    if let Some(backtrace) = config.backtrace {
        let mut attributes: HashMap<String, String> = HashMap::new();
        attributes.insert(String::from("application"), String::from("cfddns"));

        if let Ok(hostname) = gethostname().into_string() {
            attributes.insert(String::from("hostname"), hostname);
        }

        backtraceio::init(
            &backtrace.token,
            &backtrace.url,
            None,
            Some(attributes.clone()),
        );

        backtraceio::register_error_handler(
            &backtrace.url,
            &backtrace.token,
            move |r: &mut backtraceio::Report, _| {
                for (key, value) in &attributes.clone() {
                    r.attributes.insert(key.to_string(), value.to_string());
                }
            },
        );
    }

    let client = Cloudflare::init(config.api_token, config.zone_id)
        .context("Failed to initialize the cloudflare client")
        .map_err(|e| anyhow!("{:#}", e))
        .submit_error()?;

    loop {
        let public_ip = ip::get_public_ip()
            .await
            .context("Failed to fetch public ip")
            .map_err(|e| anyhow!("{:#}", e))
            .submit_error()?;
        let records = client
            .get_records()
            .await
            .context("Failed to fetch dns records")
            .map_err(|e| anyhow!("{:#}", e))
            .submit_error()?;

        // Filter records, first keeping only A records.
        // Followed by filtering out the records that don't have a public public
        // ip matching the one we got from ipify.io.
        // We then update the record to have the ip we got from ipify.
        let records_to_update = records
            .result
            .iter()
            .filter(|record| record.r#type == "A")
            .filter(|record| config.domains.contains(&record.name))
            .filter(|record| record.content != public_ip)
            .map(|record| record.to_owned())
            .map(|mut record| {
                record.content = (public_ip).to_owned();
                record
            })
            .collect::<Vec<Record>>();

        let records_from_cloudflare = records
            .result
            .iter()
            .map(|record| record.to_owned())
            .map(|record| record.name)
            .collect::<Vec<String>>();

        let records_to_create = config
            .domains
            .iter()
            .filter(|record| !records_from_cloudflare.contains(record))
            .map(|record| record.to_owned())
            .collect::<Vec<String>>();

        for record in records_to_create {
            info!("Created record: {} to point to: {}", &record, public_ip);
            client
                .create_record(&record, &public_ip)
                .await
                .context("Failed to create dns record")
                .map_err(|e| anyhow!("{:#}", e))
                .submit_error()?;
        }

        for record in records_to_update {
            info!(
                "Updated record: {} to point to: {}",
                &record.name, public_ip
            );
            client
                .update_record(record.to_owned())
                .await
                .context("Failed to update dns record")
                .map_err(|e| anyhow!("{:#}", e))
                .submit_error()?;
        }

        sleep(Duration::from_secs(config.interval.unwrap_or(300) as u64)).await;
    }
}
