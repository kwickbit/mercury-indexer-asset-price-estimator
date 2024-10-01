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
    let envelopes = client.reader().envelopes();
    let transaction_results = client.reader().tx_processing();
    let swaps_from_path_payment_offers =
        filter::swaps_from_path_payment_offers(&envelopes, &transaction_results);
    let swaps_from_path_payment_results = filter::swaps_from_path_payment_results(&envelopes, &transaction_results);
    let swaps_from_elsewhere = filter::swaps_from_elsewhere(&envelopes, &transaction_results);

    if !swaps_from_path_payment_results.is_empty() {
        client.log().debug(
            format!(
                "Swaps from path payments which give PP results: {}",
                swaps_from_path_payment_results.len()
            ),
            None,
        );
    }

    db::save_swaps(
        &client,
        &swaps_from_path_payment_offers
            .iter()
            .chain(&swaps_from_elsewhere)
            .cloned()
            .collect::<Vec<_>>(),
    );

    // If it is time, calculate and save the exchange rates from the latest sequence
    // db::save_rates(&client);
}
