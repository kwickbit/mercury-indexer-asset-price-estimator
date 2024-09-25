mod config;
mod db;
mod filter;
mod utils;

use serde::{Deserialize, Serialize};
use zephyr_sdk::EnvClient;

use db::exchange_rate;
use db::models::RatesDbRow;

#[no_mangle]
pub extern "C" fn on_close() {
    // The basics
    let client = EnvClient::new();
    let sequence = client.reader().ledger_sequence();
    let logger = create_logger(&client);

    // Hit the DB only when needed
    let should_log = sequence % config::LOGGING_INTERVAL == 0;
    let needs_exchange_rates = should_log || config::SAVE_RATES_TO_DATABASE;

    let exchange_rates = if needs_exchange_rates {
        exchange_rate::calculate_exchange_rates(&client)
    } else {
        Default::default()
    };

    if should_log {
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

    if config::SAVE_SWAPS_TO_DATABASE {
        let swaps = filter::swaps(client.reader().tx_processing());
        db::save_swaps(&client, &swaps);
        logger(&format!(
            "Sequence {sequence}, saved {} swaps to the database",
            swaps.len()
        ));
    }

    if config::SAVE_RATES_TO_DATABASE {
        let did_save_rates = db::save_rates(&client, &exchange_rates);

        if did_save_rates {
            logger(&format!(
                "Saved exchange rates of {} floatcoins to the database",
                exchange_rates.len()
            ));
        } else {
            logger("Rates were already found in the database");
        }
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
