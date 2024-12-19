mod api;
mod config;
mod db;
mod filter;
mod utils;

use db::swap::Swap;
use zephyr_sdk::EnvClient;

#[no_mangle]
pub extern "C" fn on_close() {
    // Get data from the latest ledger
    let client = EnvClient::new();
    let results = client.reader().tx_processing();
    let soroban_events = client.reader().soroban_events();

    // Extract the swaps
    let swaps = filter::swaps(results);
    let soroswap_swaps = filter::soroswap_swaps(soroban_events);

    // Save swaps
    let all_swaps = &swaps
        .clone()
        .into_iter()
        .chain(soroswap_swaps.clone())
        .collect::<Vec<Swap>>();

    db::save_swaps(&client, all_swaps);

    // If it is time, calculate and save the exchange rates
    db::save_rates(&client);
}
