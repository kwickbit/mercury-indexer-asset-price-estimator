use zephyr_sdk::{
    soroban_sdk::xdr::{
        OperationResult, OperationResultTr, TransactionResultResult::TxSuccess, VecM,
    },
    EnvClient,
};

#[no_mangle]
pub extern "C" fn on_close() {
    let env = EnvClient::new();
    let reader = env.reader();
    let sequence = reader.ledger_sequence();
    let events = reader.tx_processing();

    let successful_ops: Vec<OperationResult> = events
        .clone()
        .into_iter()
        .filter_map(|event| {
            if let TxSuccess(op_results) = event.result.result.result {
                Some(op_results.into_vec())
            } else {
                None
            }
        })
        .flatten()
        .collect();

    let payments: Vec<OperationResultTr> = events
        .into_iter()
        .filter_map(|event| {
            if let TxSuccess(op_results) = event.result.result.result {
                Some(transaction_payment_operations(op_results))
            } else {
                None
            }
        })
        .flatten()
        .collect();

    env.log().debug(
        &format!(
            "Sequence {} has {} operations, of which {} are payments",
            sequence,
            successful_ops.len(),
            payments.len()
        ),
        None,
    );
}

fn transaction_payment_operations(
    op_results: VecM<OperationResult, 4294967295>,
) -> Vec<OperationResultTr> {
    op_results
        .iter()
        .filter_map(|outer: &OperationResult| match outer {
            OperationResult::OpInner(inner) => match inner {
                OperationResultTr::Payment(_)
                | OperationResultTr::PathPaymentStrictReceive(_)
                | OperationResultTr::PathPaymentStrictSend(_) => Some(inner.clone()),
                _ => None,
            },
            _ => None,
        })
        .collect::<Vec<OperationResultTr>>()
}
