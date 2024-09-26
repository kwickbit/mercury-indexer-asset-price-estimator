mod api;
mod config;
mod db;
mod filter;
mod utils;

use zephyr_sdk::{DatabaseInteract, EnvClient};

use config::RATE_UPDATE_INTERVAL;
use db::{exchange_rate, savepoint::Savepoint};

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

    save_swaps(&client);
    save_exchange_rates(&client, exchange_rates);
    update_savepoint(&client, &logger, sequence);
}

fn create_logger(env: &EnvClient) -> impl Fn(&str) + '_ {
    move |args| env.log().debug(args, None)
}

#[allow(dead_code)]
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

fn save_swaps(client: &EnvClient) {
    if config::SAVE_SWAPS_TO_DATABASE {
        let swaps = filter::swaps(client.reader().tx_processing());
        db::save_swaps(client, &swaps);
    }
}

fn save_exchange_rates(
    client: &EnvClient,
    exchange_rates: std::collections::HashMap<String, (f64, f64)>,
) {
    if config::SAVE_RATES_TO_DATABASE {
        db::save_rates(client, &exchange_rates);
    }
}

fn update_savepoint(client: &EnvClient, logger: &impl Fn(&str), sequence: u32) {
    let savepoints: Vec<Savepoint> = client.read::<Savepoint>();
    let current_timestamp: u64 = client.reader().ledger_timestamp();

    match savepoints.len() {
        0 => {
            let savepoint = Savepoint {
                savepoint: client.reader().ledger_timestamp(),
            };

            savepoint.put(client);
            logger(&format!("First savepoint: {current_timestamp}"));
        }
        1 => {
            let latest_savepoint: u64 = savepoints[0].savepoint;

            if current_timestamp - latest_savepoint > RATE_UPDATE_INTERVAL {
                let savepoint = Savepoint {
                    savepoint: current_timestamp,
                };

                if let Err(e) = client
                    .update()
                    .column_equal_to("savepoint", latest_savepoint)
                    .execute(&savepoint)
                {
                    logger(&format!(
                        "Sequence {sequence} failed to update savepoint: {e}"
                    ))
                };
            }
        }
        _ => {
            client.log().error("More than one savepoint found!", None);
        }
    }
}
