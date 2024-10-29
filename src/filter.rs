use zephyr_sdk::soroban_sdk::xdr::{
    OperationResultTr, TransactionResultMeta, TransactionResultResult,
};

use crate::db::swap::Swap;
use crate::utils::{extract_transaction_results, get_claims_from_operation};

/**
 * We 'fish' every swap from each ledger close. There is some Vec flattening because:
 * - there are many transactions in each close;
 * - there are many operations in each transaction;
 * - there can be many swaps in each operation.
 *
 * An operation can have no swaps if it is a create account, create contract, etc.
 * If its result is an Offer or PathPayment type, it can have multiple swaps.
 */
pub(crate) fn swaps(transaction_results: Vec<TransactionResultMeta>) -> Vec<Swap> {
    transaction_results
        .iter()
        .filter(is_transaction_successful)
        .flat_map(swaps_from_transaction)
        .collect()
}

fn is_transaction_successful(transaction: &&TransactionResultMeta) -> bool {
    matches!(
        transaction.result.result.result,
        TransactionResultResult::TxSuccess(_)
    )
}

fn swaps_from_transaction(transaction: &TransactionResultMeta) -> Vec<Swap> {
    let operations = extract_transaction_results(transaction);
    operations.iter().flat_map(swaps_from_operation).collect()
}

fn swaps_from_operation(operation: &OperationResultTr) -> Vec<Swap> {
    let claims = get_claims_from_operation(operation);

    claims
        .iter()
        .filter_map(|claim| Swap::try_from(claim).ok())
        .collect()
}
