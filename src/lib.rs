mod config;
mod format;
mod transaction;
mod transaction_filter;
mod utils;

use zephyr_sdk::{
    prelude::*,
    soroban_sdk::xdr::{
        Operation, OperationResult, ScString, ScVal, TransactionEnvelope, TransactionResultMeta,
    },
    DatabaseDerive, EnvClient, EnvLogger,
};

use config::{DO_DB_STUFF, FORCE_MILESTONE, MILESTONE_INTERVAL, PRINT_TRANSACTION_DETAILS};
use format::{format_interesting_transaction, format_offer};
use transaction_filter::{is_offer_result, is_usdc_op};

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
    let envelopes = &reader.envelopes();
    let results = &reader.tx_processing();

    let events = envelopes
        .iter()
        .zip(results)
        .collect::<Vec<(&TransactionEnvelope, &TransactionResultMeta)>>();

    // Process the data
    let interesting_transactions = transaction_filter::interesting_transactions(
        &events,
        Some(is_usdc_op),
        Some(is_offer_result),
    );

    // Write to logs
    let env_logger = client.log();
    let logger = create_logger(&env_logger);

    if sequence % MILESTONE_INTERVAL == 0
        && (FORCE_MILESTONE || interesting_transactions.is_empty())
    {
        logger(&format!("Sequence {}", sequence));
    }

    if !interesting_transactions.is_empty() {
        if PRINT_TRANSACTION_DETAILS {
            interesting_transactions
                .iter()
                .enumerate()
                .for_each(|(index, transaction)| {
                    logger(&format_interesting_transaction(
                        sequence,
                        transaction,
                        index + 1,
                        interesting_transactions.len(),
                        format_offer,
                    ));
                });
        } else {
            logger(&format!(
                "Sequence {sequence} had {} interesting transactions",
                interesting_transactions.len()
            ));
        }
    }

    if !DO_DB_STUFF {
        return;
    }

    let (operations, results): (Vec<Operation>, Vec<OperationResult>) = interesting_transactions
        .into_iter()
        .flat_map(|tx| tx.operations.into_iter().zip(tx.results))
        .unzip();

    let inner_envelope = format!(
        "Sequence {sequence} had {} operations: {:#?}",
        operations.len(),
        operations
    );
    let envelope = ScVal::String(ScString((&inner_envelope).try_into().unwrap()));

    logger("Successfully built envelope");

    let inner_res_meta = format!(
        "Sequence {sequence} had {} results: {:#?}",
        results.len(),
        results.first()
    );

    logger(&format!("Inner result meta: {}", inner_res_meta));

    let res_meta = ScVal::String(ScString((&inner_res_meta).try_into().unwrap()));

    let table = StorageTable {
        is_stored: ScVal::Bool(true),
        envelope,
        res_meta,
    };

    table.put(&client);
    logger(&format!(
        "Should have written sequence {sequence} to the DB"
    ));
}

#[allow(dead_code)]
fn create_logger(env: &EnvLogger) -> impl Fn(&str) + '_ {
    move |args| env.debug(args, None)
}
