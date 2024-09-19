mod config;
// mod db;
// mod filter;
mod log;
// mod swap;
// mod utils;

use zephyr_sdk::EnvClient;
use config::DO_DB_STUFF;
// use db::do_old_db_stuff;
use log::log_milestone;

#[no_mangle]
pub extern "C" fn on_close() {
    // The Zephyr client
    let client = EnvClient::new();

    // Collect the data we need
    let reader = client.reader();
    let sequence = reader.ledger_sequence();
    let transaction_results = reader.tx_processing();
    log_milestone(&client, sequence, transaction_results.len());

    // Process the data
    // let swaps = filter::swaps(transaction_results);

    // let client_clone = client.clone();
    // let _logger = log(&client_clone, sequence, &swaps);

    if DO_DB_STUFF {
        // do_db_stuff(client, swaps);
        // do_old_db_stuff(client, sequence, transaction_results.len());
    }
}
