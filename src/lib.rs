mod config;
mod db;
mod filter;
mod swap;
mod utils;

// Note to self: qualified imports mean things I am not currently using,
// but are here because I will soon use.
use db::SwapDbRow;
use zephyr_sdk::EnvClient;

#[no_mangle]
pub extern "C" fn on_close() {
    // The Zephyr client
    let client = EnvClient::new();
    let logger = create_logger(&client);
    let rows = client.read::<SwapDbRow>();

    let floatcoin_counts = rows
        .iter()
        .fold(std::collections::HashMap::new(), |mut counts, row| {
            *counts.entry(&row.floating).or_insert(0) += 1;
            counts
        });

    let log_message = floatcoin_counts
        .iter()
        .map(|(coin, count)| format!("{} {}", count, coin))
        .collect::<Vec<_>>()
        .join(", ");

    logger(&format!("Found {} swaps. Counts by floatcoin: {}", rows.len(), log_message));

    if config::SAVE_SWAPS_TO_DATABASE {
        let swaps = filter::swaps(client.reader().tx_processing());
        db::save_swaps(&client, &swaps);
        logger(&format!("Saved {} swaps to the database", swaps.len()));
    }
}

fn create_logger(env: &EnvClient) -> impl Fn(&str) + '_ {
    move |args| env.log().debug(args, None)
}
