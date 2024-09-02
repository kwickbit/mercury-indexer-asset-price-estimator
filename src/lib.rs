use zephyr_sdk::{soroban_sdk::xdr::TransactionResultResult::TxSuccess, EnvClient};

#[no_mangle]
pub extern "C" fn on_close() {
    let env = EnvClient::new();
    let reader = env.reader();
    let sequence = reader.ledger_sequence();

    let events = reader.tx_processing();
    if let TxSuccess(op_results) = &events.first().unwrap().result.result.result {
        env.log().debug(
            format!(
                "Sequence {sequence} suceeded.\nThere were {} operation results in it.\n",
                op_results.len()
            ),
            None,
        );
        // We print the first operation result, if it exists
        if let Some(op_result) = op_results.first() {
            env.log()
                .debug(format!("First operation result: {op_result:?}",), None)
        }
    } else {
        let message = format!("The first event in sequence {sequence} was not successful.");
        env.log().debug(message, None);
    }
}
