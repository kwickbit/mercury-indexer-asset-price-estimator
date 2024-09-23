mod config;
// mod db;
mod filter;
// mod log;
mod swap;
mod utils;

use config::DO_DB_STUFF;
use zephyr_sdk::{EnvClient, EnvLogger};
// use db::do_db_stuff;
// use log::log;

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
    logger(&format!("In sequence {sequence}, processed {} swaps", swaps.len()));

    if DO_DB_STUFF {
        // do_db_stuff(client, swaps);
    }
}

fn create_logger(env: &EnvLogger) -> impl Fn(&str) + '_ {
    move |args| env.debug(args, None)
}
