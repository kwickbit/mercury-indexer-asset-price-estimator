mod config;
mod db;
mod exchange_rate;
mod filter;
mod swap;
mod utils;

use config::{LOGGING_INTERVAL, SAVE_SWAPS_TO_DATABASE};
use db::save_swaps;
use serde::{Deserialize, Serialize};
use zephyr_sdk::EnvClient;

#[no_mangle]
pub extern "C" fn on_close() {
    let client = EnvClient::new();
    let sequence = client.reader().ledger_sequence();
    let logger = create_logger(&client);

    if sequence % LOGGING_INTERVAL == 0 {
        let exchange_rates = exchange_rate::calculate_exchange_rates(&client);

        let log_message = exchange_rates
            .iter()
            .map(|(coin, (weighted_average, volume))| {
                format!("{coin}: {weighted_average:.4} (volume ${volume:.2})")
            })
            .collect::<Vec<_>>()
            .join(", ");

        logger(&format!(
            "Sequence {sequence}, exchange rates: {log_message}"
        ));
    }

    if SAVE_SWAPS_TO_DATABASE {
        let swaps = filter::swaps(client.reader().tx_processing());
        save_swaps(&client, &swaps);
        logger(&format!("Saved {} swaps to the database", swaps.len()));
    }
}

fn create_logger(env: &EnvClient) -> impl Fn(&str) + '_ {
    move |args| env.log().debug(args, None)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    asset: String,
}

#[no_mangle]
pub extern "C" fn get_exchange_rate() {
    let client = EnvClient::empty();
    let request: Request = client.read_request_body();
    let exchange_rates = exchange_rate::calculate_exchange_rates(&client);

    match exchange_rates.get(&request.asset) {
        Some(&(rate, _)) => {
            let response = serde_json::json!({
                "asset": request.asset,
                "exchange_rate": rate.to_string(),
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
