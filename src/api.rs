use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zephyr_sdk::EnvClient;

use crate::db::exchange_rate::RatesDbRow;

#[derive(Serialize, Deserialize)]
pub struct ExchangeRateRequest {
    asset: String,
}

#[no_mangle]
pub extern "C" fn get_exchange_rate() {
    let client = EnvClient::empty();
    let request = client.read_request_body::<ExchangeRateRequest>();
    let exchange_rates = client.read::<RatesDbRow>();

    let requested_asset_data = exchange_rates
        .iter()
        .find(|row| row.floating == request.asset);

    match requested_asset_data {
        Some(data) => {
            let response = serde_json::json!({
                "asset": request.asset,
                "exchange_rate": data.rate.to_string(),
            });
            client.conclude(&response);
        }
        None => {
            let response = serde_json::json!({
                "asset": request.asset,
                "error": "Asset not found",
            });
            client.conclude(&response);
        }
    }
}

#[no_mangle]
pub extern "C" fn get_all_exchange_rates() {
    let client = EnvClient::empty();
    let exchange_rates = client.read::<RatesDbRow>();

    let response = serde_json::json!(exchange_rates.iter().fold(
        HashMap::new(),
        |mut acc: HashMap<String, HashMap<String, String>>, row| {
            acc.entry(row.floating.clone())
                .or_default()
                .insert(row.timestamp.unwrap().to_string(), row.rate.to_string());
            acc
        }
    ));

    client.conclude(&response);
}
