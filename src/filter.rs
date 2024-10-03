use zephyr_sdk::soroban_sdk::xdr::{
    ManageBuyOfferResult, ManageOfferSuccessResultOffer, ManageSellOfferResult, OperationResultTr,
    PathPaymentStrictReceiveResult, PathPaymentStrictSendResult, TransactionResultMeta,
    TransactionResultResult,
};

use crate::config::USDC;
use crate::db::swap::Swap;
use crate::utils::extract_transaction_results;

pub fn path_payment_results(results: &[TransactionResultMeta]) -> Vec<OperationResultTr> {
    results
        .iter()
        .flat_map(|result| {
            let operation_results = extract_transaction_results(result);
            operation_results.into_iter().filter(is_path_payment_result)
        })
        .collect()
}

fn is_path_payment_result(operation_result: &OperationResultTr) -> bool {
    matches!(
        operation_result,
        OperationResultTr::PathPaymentStrictReceive(PathPaymentStrictReceiveResult::Success(_))
            | OperationResultTr::PathPaymentStrictSend(PathPaymentStrictSendResult::Success(_))
    )
}

pub fn swaps(transaction_results: Vec<TransactionResultMeta>) -> Vec<Swap> {
    transaction_results
        .iter()
        .filter(|transaction_result| is_successful(transaction_result))
        .filter_map(build_swaps)
        .flatten()
        .collect::<Vec<Swap>>()
}

fn is_successful(result_meta: &TransactionResultMeta) -> bool {
    matches!(
        result_meta.result.result.result,
        TransactionResultResult::TxSuccess(_)
    )
}

fn build_swaps(transaction_result: &TransactionResultMeta) -> Option<Vec<Swap>> {
    let operation_results = extract_transaction_results(transaction_result);
    // It could be that a transaction does not contain swaps (e.g. a simple payment,
    // an account creation).
    let potential_swaps: Vec<Swap> = operation_results.iter().filter_map(build_swap).collect();
    (!potential_swaps.is_empty()).then_some(potential_swaps)
}

fn build_swap(operation_result: &OperationResultTr) -> Option<Swap> {
    match operation_result {
        // At the moment, these are the only swaps we can reliably detect.
        // Later we might also get swaps from PathPayments.
        OperationResultTr::ManageSellOffer(ManageSellOfferResult::Success(offer_result))
        | OperationResultTr::CreatePassiveSellOffer(ManageSellOfferResult::Success(offer_result))
        | OperationResultTr::ManageBuyOffer(ManageBuyOfferResult::Success(offer_result)) => {
            match &offer_result.offer {
                ManageOfferSuccessResultOffer::Created(offer_entry)
                | ManageOfferSuccessResultOffer::Updated(offer_entry)
                    // Some swaps involve only floating assets; we are not interested in those.
                    if offer_entry.selling == USDC || offer_entry.buying == USDC =>
                {
                    Some(Swap::from(offer_entry.clone()))
                }
                _ => None,
            }
        }
        _ => None,
    }
}
