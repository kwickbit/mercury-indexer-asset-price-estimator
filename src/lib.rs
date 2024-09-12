mod exchange_rate;
mod format;
mod transaction;
mod transaction_filter;
mod utils;

use format::format_claim_atom;
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
    let claim_atoms = transaction_filter::get_claim_atoms(&events);

    // Write to logs
    let env_logger = client.log();
    let logger = create_logger(&env_logger);

    if claim_atoms.is_empty() && sequence % 12 == 0 {
        logger(&format!("Sequence {}", sequence));
    }

    if !claim_atoms.is_empty() {
        claim_atoms
            .iter()
            .enumerate()
            .for_each(|(index, claim_atom)| {
                logger(&format!("Claim #{}", index + 1));
                logger(&format_claim_atom(claim_atom));
            });
    }
}

fn create_logger(env: &EnvLogger) -> impl Fn(&str) + '_ {
    move |args| env.debug(args, None)
}
