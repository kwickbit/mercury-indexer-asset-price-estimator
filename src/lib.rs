mod config;
mod db;
mod filter;
mod swap;
mod utils;

use zephyr_sdk::{EnvClient, EnvLogger};
use db::save_swaps;

#[no_mangle]
pub extern "C" fn on_close() {
    // The Zephyr client
    let client = EnvClient::new();

    // Collect the data we need
    let reader = client.reader();
    let sequence = reader.ledger_sequence();

    // Process the data
    let env_logger = client.log();
    let logger = create_logger(&env_logger);
    let swaps = filter::swaps(reader.tx_processing());
    logger(&format!(
        "In sequence {sequence}, processed {} swaps",
        swaps.len()
    ));

    save_swaps(client, swaps);
}

fn create_logger(env: &EnvLogger) -> impl Fn(&str) + '_ {
    move |args| env.debug(args, None)
}
