//! Consume data from the Stellar blockchain to output asset prices in USD
//!
//! Harnesses the Mercury indexer to identify asset swaps involving the USDC
//! stablecoin. Each ledger close is scanned for Soroban DEX events and classic
//! path payment transactions to identify swaps. Every hour, swaps are
//! distilled into a single exchange rate per asset. Mercury serverless
//! functions allow querying the DB for single exchange rates or batches.

#![warn(missing_docs)]

mod api;
mod config;
mod db;
mod filter;
mod utils;

use db::swap::Swap;
use zephyr_sdk::EnvClient;

/// Processes events from the latest ledger close to track and calculate asset exchange rates.
///
/// This function:
/// 1. Retrieves transaction and Soroban event data from the latest ledger
/// 2. Extracts swap events from both classic and Soroswap transactions
/// 3. Saves the swap data to the database
/// 4. Periodically calculates and saves exchange rates based on accumulated swap data
///
/// Called automatically by the Mercury indexer on each ledger close.
#[no_mangle]
pub extern "C" fn on_close() {
    let client = EnvClient::new();
    let results = client.reader().tx_processing();
    let soroban_events = client.reader().soroban_events();
    let swaps = filter::swaps(results);
    let soroswap_swaps = filter::soroswap_swaps(soroban_events);

    let all_swaps = &swaps
        .clone()
        .into_iter()
        .chain(soroswap_swaps.clone())
        .collect::<Vec<Swap>>();

    db::save_swaps(&client, all_swaps);
    db::save_rates(&client);
}
