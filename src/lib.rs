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
    let transaction_results = reader.tx_processing();

    // Process the data
    let env_logger = client.log();
    let logger = create_logger(&env_logger);
    logger(&format!("==> Sequence {sequence}"));
    let swaps = filter::swaps(transaction_results, &logger);
    logger(&format!("--> Processed {} swaps", swaps.len()));

    if DO_DB_STUFF {
        // do_db_stuff(client, swaps);
    }
}

fn create_logger(env: &EnvLogger) -> impl Fn(&str) + '_ {
    move |args| env.debug(args, None)
}
