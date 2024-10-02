mod api;
mod config;
mod db;
mod filter;
mod utils;

use zephyr_sdk::EnvClient;

#[no_mangle]
pub extern "C" fn on_close() {
    // The basics
    let client = EnvClient::new();

    // Read and save the swaps from the latest sequence
    let swaps = filter::swaps(client.reader().tx_processing());
    db::save_swaps(&client, &swaps);

    // If it is time, calculate and save the exchange rates from the latest sequence
    // db::save_rates(&client);
}
