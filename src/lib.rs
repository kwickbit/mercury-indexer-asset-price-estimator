mod config;
mod db;
mod filter;
mod swap;
mod utils;

// Note to self: qualified imports mean things I am not currently using,
// but are here because I will soon use.
use db::SwapDbRow;
use swap::Swap;
use zephyr_sdk::{EnvClient, EnvLogger};

#[no_mangle]
pub extern "C" fn on_close() {
    // The Zephyr client
    let client = EnvClient::new();

    let rows = client.read::<SwapDbRow>();
    let first_row: Swap = SwapDbRow::try_into(rows.first().unwrap().clone()).unwrap();
    client.log().debug(
        &format!("Found {} swaps. First one: {first_row}", rows.len()),
        None,
    );

    if config::SAVE_SWAPS_TO_DATABASE {
        let swaps = filter::swaps(client.reader().tx_processing());
        db::save_swaps(&client, &swaps);
        create_logger(&client.log())(&format!("Saved {} swaps to the database", swaps.len()));
    }
}

fn create_logger(env: &EnvLogger) -> impl Fn(&str) + '_ {
    move |args| env.debug(args, None)
}
