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

    if !soroswap_swaps.is_empty() {
        let swaps_string = soroswap_swaps
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(" // ");

        client.log().debug(
            format!(
                "Sequence #{}: {swaps_string}",
                client.reader().ledger_sequence(),
            ),
            None,
        );
    }

    // Save swaps and, if it is time, calculate and save the exchange rates
    let all_swaps = &swaps
        .clone()
        .into_iter()
        .chain(soroswap_swaps.clone())
        .collect::<Vec<Swap>>();

    client.log().debug(
        format!(
            "Saving {} swaps: {} classic and {} Soroswap",
            all_swaps.len(),
            swaps.len(),
            soroswap_swaps.len()
        ),
        None,
    );

    db::save_swaps(&client, all_swaps);
    db::save_rates(&client);
}
