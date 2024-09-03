use zephyr_sdk::{
    soroban_sdk::xdr::{OperationResult, OperationResultCode, TransactionResultResult::TxSuccess},
    EnvClient,
};

#[no_mangle]
pub extern "C" fn on_close() {
    let env = EnvClient::new();
    let reader = env.reader();
    let sequence = reader.ledger_sequence();
    // let events = reader.tx_processing();

    env.log().debug(
        format!("Sequence {}", sequence),
        // &format!("Sequence {} has {} events", sequence, events.len()),
        None,
    );

    // let successful_operations: Vec<OperationResult> = events
    //     .into_iter()
    //     .filter_map(|event| {
    //         let result_pair = event.result;
    //         let transaction_result = result_pair.result;
    //         let transaction_result_result_yes_really: zephyr_sdk::soroban_sdk::xdr::TransactionResultResult = transaction_result.result;

    //         if let TxSuccess(op_results) = transaction_result_result_yes_really {
    //             Some(
    //                 op_results
    //                     .iter()
    //                     .filter(|op_result| op_result.discriminant() == OperationResultCode::OpInner)
    //                     .cloned()
    //                     .collect::<Vec<_>>())
    //         } else {
    //             None
    //         }
    //     })
    //     .flatten()
    //     .collect();

    // env.log().debug(
    //     &format!(
    //         "On sequence {} there are {} successful operations",
    //         sequence,
    //         successful_operations.len()
    //     ),
    //     None,
    // );

    // env.log()
    //     .debug(&format!("{:.280?}...", successful_operations), None)
}
