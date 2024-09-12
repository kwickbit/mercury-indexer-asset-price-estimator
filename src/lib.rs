mod exchange_rate;
mod format;
mod transaction;
mod transaction_filter;
mod utils;

use format::{format_interesting_transaction, format_path_payment};
use zephyr_sdk::{EnvClient, EnvLogger};

#[no_mangle]
pub extern "C" fn on_close() {
    // The Zephyr client
    let client = EnvClient::new();

    // Collect the data we need
    let reader = client.reader();
    let sequence = reader.ledger_sequence();
    let events = reader.envelopes_with_meta();

    // Process the data
    let interesting_transactions = transaction_filter::interesting_transactions(&events);

    // Write to logs
    let env_logger = client.log();
    let logger = create_logger(&env_logger);

    if interesting_transactions.is_empty() && sequence % 12 == 0 {
        logger(&format!("Sequence {}", sequence));
    }

    if !interesting_transactions.is_empty() {
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
                // let rates = transaction.exchange_rates(&client);
                // if !rates.is_empty() {
                //     rates.into_iter().for_each(|rate| {
                //         logger(&format!("{}: ${:.6}", rate.asset, rate.usd_value));
                //     });
                // }
            });
    }
}

fn create_logger(env: &EnvLogger) -> impl Fn(&str) + '_ {
    move |args| env.debug(args, None)
}
