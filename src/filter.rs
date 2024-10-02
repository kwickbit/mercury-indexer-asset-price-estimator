use zephyr_sdk::soroban_sdk::xdr::{
    ManageBuyOfferResult, ManageOfferSuccessResultOffer, ManageSellOfferResult, OperationResultTr,
    TransactionResultMeta, TransactionResultResult,
};
use zephyr_sdk::EnvClient;

use crate::config::USDC;
use crate::db::swap::Swap;
use crate::utils::extract_transaction_results;

pub fn swaps(transaction_results: Vec<TransactionResultMeta>, client: &EnvClient) -> Vec<Swap> {
    transaction_results
        .iter()
        .filter(|transaction_result| is_successful(transaction_result))
        .filter_map(|result| build_swaps(result, client))
        .flatten()
        .collect::<Vec<Swap>>()
}

fn is_successful(result_meta: &TransactionResultMeta) -> bool {
    matches!(
        result_meta.result.result.result,
        TransactionResultResult::TxSuccess(_)
    )
}

fn build_swaps(
    transaction_result: &TransactionResultMeta,
    client: &EnvClient,
) -> Option<Vec<Swap>> {
    let operation_results = extract_transaction_results(transaction_result);
    // It could be that a transaction does not contain swaps (e.g. a simple payment,
    // an account creation).
    let potential_swaps: Vec<Swap> = operation_results
        .iter()
        .filter_map(|result| build_swap(result, client))
        .collect();
    (!potential_swaps.is_empty()).then_some(potential_swaps)
}

fn build_swap(operation_result: &OperationResultTr, client: &EnvClient) -> Option<Swap> {
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
