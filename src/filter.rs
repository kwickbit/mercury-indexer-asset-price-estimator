use zephyr_sdk::soroban_sdk::xdr::{
    ManageBuyOfferResult, ManageOfferSuccessResultOffer, ManageSellOfferResult, OperationResultTr,
    TransactionResultMeta, TransactionResultResult,
};

use crate::config::ALLOW_UNSUCCESSFUL_TRANSACTIONS;
use crate::swap::Swap;
use crate::utils::{extract_transaction_results, is_stablecoin};

pub fn swaps(transaction_results: Vec<TransactionResultMeta>) -> Vec<Swap> {
    transaction_results
        .iter()
        .filter(|transaction_result| is_successful(transaction_result))
        .filter_map(build_swaps)
        .flatten()
        .collect::<Vec<Swap>>()
}

fn is_successful(result_meta: &TransactionResultMeta) -> bool {
    ALLOW_UNSUCCESSFUL_TRANSACTIONS
        || matches!(
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
                    if is_stablecoin(&offer_entry.selling) || is_stablecoin(&offer_entry.buying) =>
                {
                    Some(Swap::from(offer_entry.clone()))
                }
                _ => None,
            }
        }
        _ => None,
    }
}
