use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use zephyr_sdk::EnvClient;

use crate::db::exchange_rate::RatesDbRow;

#[derive(Serialize, Deserialize)]
pub struct ExchangeRateRequest {
    asset: String,
}

#[no_mangle]
pub extern "C" fn get_exchange_rate() {
    let client = EnvClient::empty();
    let request: ExchangeRateRequest = client.read_request_body();
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

    let response = serde_json::json!(exchange_rates
        .iter()
        .map(|row| (row.floating.clone(), format!("{:.4}", row.rate)))
        .collect::<HashMap<_, _>>());

    client
        .log()
        .debug(&format!("V3 response: {}", response), None);

    client.conclude(&response);
}
