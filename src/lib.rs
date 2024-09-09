mod formatting;
mod transaction;
mod transaction_filter;
mod utils;

use transaction::InterestingTransaction;
use zephyr_sdk::{EnvClient, EnvLogger};

#[no_mangle]
pub extern "C" fn on_close() {
    // The Zephyr client
    let client = EnvClient::new();

    // We collect the data we need
    let reader = client.reader();
    let sequence = reader.ledger_sequence();
    let events = reader.envelopes_with_meta();

    // Process the data
    let interesting_transactions: Vec<InterestingTransaction> =
        transaction_filter::interesting_transactions(&events);

    // Write to logs
    let env_logger = client.log();
    let logger = create_logger(&env_logger);

    if interesting_transactions.is_empty() {
        if sequence % 12 == 0 {
            logger(&format!("-- Sequence {} --", sequence));
        }
    } else {
        let formatted_transactions =
            formatting::format_interesting_transactions(&interesting_transactions);

        formatted_transactions
            .into_iter()
            .for_each(|formatted_transaction| logger(&formatted_transaction));
    }
}

fn create_logger(env: &EnvLogger) -> impl Fn(&str) + '_ {
    move |args| env.debug(args, None)
}
