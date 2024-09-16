mod config;
mod format;
mod transaction;
mod transaction_filter;
mod utils;

use config::{FORCE_MILESTONE, MILESTONE_INTERVAL};
use format::{format_interesting_transaction, format_path_payment};
use zephyr_sdk::{
    soroban_sdk::xdr::{TransactionEnvelope, TransactionResultMeta},
    EnvClient, EnvLogger,
};

#[no_mangle]
pub extern "C" fn on_close() {
    // The Zephyr client
    let client = EnvClient::new();

    // Collect the data we need
    let reader = client.reader();
    let sequence = reader.ledger_sequence();
    let envelopes = reader.envelopes();
    let results = reader.tx_processing();

    let events = envelopes
        .into_iter()
        .zip(results)
        .collect::<Vec<(TransactionEnvelope, TransactionResultMeta)>>();

    // Process the data
    let interesting_transactions = transaction_filter::interesting_transactions(&events);

    // Write to logs
    let env_logger = client.log();
    let logger = create_logger(&env_logger);

    if sequence % MILESTONE_INTERVAL == 0
        && (FORCE_MILESTONE || interesting_transactions.is_empty())
    {
        logger(&format!("Sequence {}", sequence));
    }

    if !interesting_transactions.is_empty() {
        logger(&format!(
            "Sequence {} with {} transactions: ",
            sequence,
            interesting_transactions.len()
        ));
        interesting_transactions
            .iter()
            .enumerate()
            .for_each(|(index, transaction)| {
                logger(
                    &format_interesting_transaction(
                        sequence,
                        transaction,
                        index + 1,
                        format_path_payment,
                    )
                    .to_string(),
                );
            });
    }
}

fn create_logger(env: &EnvLogger) -> impl Fn(&str) + '_ {
    move |args| env.debug(args, None)
}
