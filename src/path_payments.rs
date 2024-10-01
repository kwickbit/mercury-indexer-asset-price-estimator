use zephyr_sdk::soroban_sdk::xdr::{
    OperationBody, OperationResultTr, TransactionEnvelope, TransactionResultMeta,
};

use crate::utils;

pub fn path_payment_stats(
    transactions: &[TransactionEnvelope],
    results: &[TransactionResultMeta],
) -> (usize, usize, usize, usize) {
    transactions.iter().zip(results).fold((0, 0, 0, 0), counter)
}

fn counter(
    counts: (usize, usize, usize, usize),
    (transaction, result): (&TransactionEnvelope, &TransactionResultMeta),
) -> (usize, usize, usize, usize) {
    let (path_payments, path_payment_ops, path_payment_results, offer_results) = counts;
    let new_path_payments = count_path_payment_ops(transaction);

    if new_path_payments > 0 {
        let (new_path_payment_results, new_offer_results) = count_results(result);
        (
            path_payments + 1,
            path_payment_ops + new_path_payments,
            path_payment_results + new_path_payment_results,
            offer_results + new_offer_results,
        )
    } else {
        counts
    }
}

fn count_path_payment_ops(transaction: &TransactionEnvelope) -> usize {
    let operations = crate::utils::extract_transaction_operations(transaction);

    operations
        .iter()
        // `count_by` would merge the filter and the count, but
        // it's not available in stable Rust yet.
        .filter(|operation| {
            matches!(
                operation,
                OperationBody::PathPaymentStrictReceive(_)
                    | OperationBody::PathPaymentStrictSend(_)
            )
        })
        .count()
}

fn count_results(result: &TransactionResultMeta) -> (usize, usize) {
    let op_results = utils::extract_transaction_results(result);

    op_results.iter().fold(
        (0, 0),
        |(path_payments, offers), op_result| match op_result {
            OperationResultTr::PathPaymentStrictReceive(_)
            | OperationResultTr::PathPaymentStrictSend(_) => (path_payments + 1, offers),
            OperationResultTr::ManageSellOffer(_)
            | OperationResultTr::ManageBuyOffer(_)
            | OperationResultTr::CreatePassiveSellOffer(_) => (path_payments, offers + 1),
            _ => (path_payments, offers),
        },
    )
}
