use crate::config;
use crate::models::{President, Presidents};

use actix_web::client::Client;
use serde::Deserialize;

///
/// Client for working with Airtables
///

#[derive(Debug, Deserialize)]
struct ResponseItem {
    id: String,
    fields: President,
}

#[derive(Debug, Deserialize)]
struct Payload {
    records: Vec<ResponseItem>,
}

async fn get_airtables_data(client: &Client) -> Vec<President> {
    let cfg = config::from_envvar();
    // Initialize presidents return data struct
    let mut presidents = vec![];

    // Create request builder and send request
    log::info!("Calling Airtables API");
    let response = client
        .get(cfg.data_load_uri)
        .header("User-Agent", "nothing-here/1.0") // "actix-web/3.0")
        .header("Authorization", format!("Bearer {}", cfg.api_key))
        .send()
        .await;

    match response {
        Ok(mut resp) => {
            let json_content: Payload = resp.json().limit(250_000).await.unwrap();

            for item in json_content.records {
                let president = item.fields;
                presidents.push(president);
            }
        }
        Err(err) => {
            log::warn!("Server Error, could not get data from data store");
            log::error!("Fatal! {}", err);
        }
    };

    presidents
}

pub async fn reload_airtables(presidents: &mut Presidents) {
    let client = Client::default();
    let data: Vec<President> = get_airtables_data(&client).await;

    for president in data {
        presidents.insert(president.short_name(), president);
    }
}
