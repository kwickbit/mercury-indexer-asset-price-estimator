mod config;
mod db;
mod filter;
mod swap;
mod utils;

// Note to self: qualified imports mean things I am not currently using,
// but are here because I will soon use.
use db::SwapDbRow;
use swap::Swap;
use zephyr_sdk::EnvClient;

#[no_mangle]
pub extern "C" fn on_close() {
    // The Zephyr client
    let client = EnvClient::new();
    let logger = create_logger(&client);

    let rows = client.read::<SwapDbRow>();
    let first_row: Swap = rows.first().unwrap().into();

    logger(&format!(
        "Found {} swaps. First one: {first_row}",
        rows.len()
    ));

    if config::SAVE_SWAPS_TO_DATABASE {
        let swaps = filter::swaps(client.reader().tx_processing());
        db::save_swaps(&client, &swaps);
        logger(&format!("Saved {} swaps to the database", swaps.len()));
    }
}

fn create_logger(env: &EnvClient) -> impl Fn(&str) + '_ {
    move |args| env.log().debug(args, None)
}
