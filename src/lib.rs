mod api;
mod config;
mod db;
mod filter;
mod utils;

use std::collections::HashMap;

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

    log(should_log, &exchange_rates, &logger, sequence);
    save_swaps(&client, &logger, sequence);
    save_exchange_rates(&client, exchange_rates, &logger);
}

fn create_logger(env: &EnvClient) -> impl Fn(&str) + '_ {
    move |args| env.log().debug(args, None)
}

fn log(
    should_log: bool,
    exchange_rates: &std::collections::HashMap<String, (f64, f64)>,
    logger: &impl Fn(&str),
    sequence: u32,
) {
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
}

fn save_swaps(client: &EnvClient, logger: &impl Fn(&str), sequence: u32) {
    if config::SAVE_SWAPS_TO_DATABASE {
        let swaps = filter::swaps(client.reader().tx_processing());
        db::save_swaps(client, &swaps);
        logger(&format!(
            "Sequence {sequence}, saved {} swaps to the database",
            swaps.len()
        ));
    }
}

fn save_exchange_rates(
    client: &EnvClient,
    exchange_rates: std::collections::HashMap<String, (f64, f64)>,
    logger: &impl Fn(&str),
) {
    if config::SAVE_RATES_TO_DATABASE {
        let did_save_rates = db::save_rates(client, &exchange_rates);

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
