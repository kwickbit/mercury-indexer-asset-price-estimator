mod api;
mod config;
mod db;
mod filter;
mod utils;

use zephyr_sdk::EnvClient;

#[no_mangle]
/**
 * On every ledger close, we read the swaps from the latest sequence and save
 * them. Once a configured time window has passed, we calculate exchange rates
 * based on those swaps and save them.
 */
pub extern "C" fn on_close() {
    // The basics
    let client = EnvClient::new();

    // Read and save the swaps from the latest sequence
    let results = client.reader().tx_processing();
    let swaps = filter::swaps(results);
    db::save_swaps(&client, &swaps);

    // If it is time, calculate and save the exchange rates from the latest sequence
    db::save_rates(&client);
}
