mod filter_transactions;
mod logger;
mod utils;

use zephyr_sdk::EnvClient;

#[no_mangle]
pub extern "C" fn on_close() {
    // The Zephyr client
    let client = EnvClient::new();

    // We collect the data we need
    let reader = client.reader();
    let sequence = reader.ledger_sequence();
    let events = reader.envelopes_with_meta();

    // Process the data
    let successful = filter_transactions::successful_usdc_txns(&events);

    // Write to logs
    let env_logger = client.log();
    let logger = logger::log(&env_logger);

    if successful.is_empty() && sequence % 12 == 0 {
        logger(&format!("-- Sequence {} --", sequence));
    }

    if !successful.is_empty() {
        logger(&format!(
            "Sequence {} has {} successful {} transactions:\n{}",
            sequence,
            successful.len(),
            utils::ASSET,
            logger::log_usdc_transactions(&successful),
        ));
    }
}
