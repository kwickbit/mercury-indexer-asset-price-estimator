mod config;
mod db;
mod exchange_rate;
mod filter;
mod swap;
mod utils;

// Note to self: qualified imports mean things I am not currently using,
// but are here because I will soon use.
use zephyr_sdk::EnvClient;

#[no_mangle]
pub extern "C" fn on_close() {
    // The Zephyr client
    let client = EnvClient::new();
    let logger = create_logger(&client);
    let exchange_rates = exchange_rate::calculate_exchange_rates(&client);

    let log_message = exchange_rates
        .iter()
        .map(|(coin, (weighted_average, volume))| {
            format!("{coin}: {weighted_average:.4} (volume ${volume:.2})")
        })
        .collect::<Vec<_>>()
        .join(", ");

    logger(&format!("Exchange rates by floatcoin: {}", log_message));

    if config::SAVE_SWAPS_TO_DATABASE {
        let swaps = filter::swaps(client.reader().tx_processing());
        db::save_swaps(&client, &swaps);
        logger(&format!("Saved {} swaps to the database", swaps.len()));
    }
}

fn create_logger(env: &EnvClient) -> impl Fn(&str) + '_ {
    move |args| env.log().debug(args, None)
}
