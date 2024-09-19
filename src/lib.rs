mod config;
mod format;
mod swap;
mod transaction;
mod transaction_filter;
mod utils;

use zephyr_sdk::{
    prelude::*,
    soroban_sdk::xdr::{ScString, ScVal},
    DatabaseDerive, EnvClient, EnvLogger,
};

use config::{DO_DB_STUFF, FORCE_MILESTONE, MILESTONE_INTERVAL, PRINT_TRANSACTION_DETAILS};
use format::format_swap;

#[derive(DatabaseDerive, Clone)]
#[with_name("storage")]
struct StorageTable {
    is_stored: ScVal,
    envelope: ScVal,
    res_meta: ScVal,
}

impl Default for StorageTable {
    fn default() -> Self {
        Self {
            is_stored: ScVal::Bool(false),
            envelope: ScVal::String(ScString("".try_into().unwrap())),
            res_meta: ScVal::String(ScString("".try_into().unwrap())),
        }
    }
}

#[no_mangle]
#[allow(unreachable_code)]
pub extern "C" fn on_close() {
    // The Zephyr client
    let client = EnvClient::new();

    // Collect the data we need
    let reader = client.reader();
    let sequence = reader.ledger_sequence();
    let transaction_results = reader.tx_processing();

    // Process the data
    let swaps = transaction_filter::swaps(transaction_results);

    // Write to logs
    let env_logger = client.log();
    let logger = create_logger(&env_logger);

    if sequence % MILESTONE_INTERVAL == 0 && (FORCE_MILESTONE || swaps.is_empty()) {
        logger(&format!("Sequence {}", sequence));
    }

    if !swaps.is_empty() {
        if PRINT_TRANSACTION_DETAILS {
            swaps.iter().for_each(|swap| {
                logger(&format_swap(swap));
            });
        } else {
            logger(&format!(
                "Sequence {sequence} had {} interesting transactions",
                swaps.len()
            ));
        }
    }

    if !DO_DB_STUFF {
        todo!();
    }

    // let (operations, results): (Vec<Operation>, Vec<OperationResult>) = swaps
    //     .into_iter()
    //     .flat_map(|tx| tx.operations.into_iter().zip(tx.results))
    //     .unzip();

    // let inner_envelope = format!(
    //     "Sequence {sequence} had {} operations: {:#?}",
    //     operations.len(),
    //     operations
    // );
    // let envelope = ScVal::String(ScString((&inner_envelope).try_into().unwrap()));

    // logger("Successfully built envelope");

    // let inner_res_meta = format!(
    //     "Sequence {sequence} had {} results: {:#?}",
    //     results.len(),
    //     results.first()
    // );

    // logger(&format!("Inner result meta: {}", inner_res_meta));

    // let res_meta = ScVal::String(ScString((&inner_res_meta).try_into().unwrap()));

    // let table = StorageTable {
    //     is_stored: ScVal::Bool(true),
    //     envelope,
    //     res_meta,
    // };

    // table.put(&client);
    // logger(&format!(
    //     "Should have written sequence {sequence} to the DB"
    // ));
}

#[allow(dead_code)]
fn create_logger(env: &EnvLogger) -> impl Fn(&str) + '_ {
    move |args| env.debug(args, None)
}
