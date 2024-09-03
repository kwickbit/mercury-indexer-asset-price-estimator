use zephyr_sdk::{
    soroban_sdk::xdr::{
        OperationResult, OperationResultTr, TransactionResult, TransactionResultMeta,
        TransactionResultPair,
        TransactionResultResult::{self, TxSuccess},
        VecM,
    },
    EnvClient,
};

#[no_mangle]
pub extern "C" fn on_close() {
    let env = EnvClient::new();
    let reader = env.reader();
    let sequence = reader.ledger_sequence();
    let events: Vec<TransactionResultMeta> = reader.tx_processing();

    let successful_operations: Vec<OperationResultTr> = events
        .into_iter()
        .filter_map(event_successful_operations)
        .flatten()
        .collect::<Vec<OperationResultTr>>();

    env.log().debug(
        &format!(
            "Sequence {} has {} payment operations",
            sequence,
            successful_operations.len()
        ),
        None,
    );

    let formatted = format!("{:?}", successful_operations.first());
    // let message = formatted.chars().take(1024).collect::<String>() + "...";

    env.log().debug(formatted, None)
}

fn event_successful_operations(event: TransactionResultMeta) -> Option<Vec<OperationResultTr>> {
    let result_pair: TransactionResultPair = event.result;
    let transaction_result: TransactionResult = result_pair.result;
    let transaction_result_result: TransactionResultResult = transaction_result.result;

    if let TxSuccess(op_results) = transaction_result_result {
        Some(transaction_operations(op_results))
    } else {
        None
    }
}

fn transaction_operations(op_results: VecM<OperationResult, 4294967295>) -> Vec<OperationResultTr> {
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
