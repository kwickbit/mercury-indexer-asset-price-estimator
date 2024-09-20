use zephyr_sdk::soroban_sdk::xdr::{
    ManageBuyOfferResult, ManageOfferSuccessResultOffer, ManageSellOfferResult, OperationResultTr,
    TransactionResultMeta, TransactionResultResult,
};

use crate::config::ALLOW_UNSUCCESSFUL_TRANSACTIONS;
use crate::swap::Swap;
use crate::utils::{extract_transaction_results, format_asset, is_stablecoin};

pub fn swaps(transaction_results: Vec<TransactionResultMeta>, logger: impl Fn(&str)) -> Vec<Swap> {
    let result: Vec<Swap> = transaction_results
        .iter()
        .filter(|transaction_result| is_successful(transaction_result))
        .enumerate()
        .filter_map(|(count, tr)| {
            let swaps = build_swaps(tr, &logger);
            if !swaps.clone()?.is_empty() {
                logger(&format!(
                    "Built {} swaps for transaction {}. The first swap is: {}",
                    swaps.as_ref().unwrap().len(),
                    count + 1,
                    swaps.clone()?.first().unwrap()
                ));
            };
            swaps
        })
        .flatten()
        .collect();

    logger(&format!("Processed into {} total swaps", result.len()));
    result
}

fn build_swaps(
    transaction_result: &TransactionResultMeta,
    _logger: &impl Fn(&str),
) -> Option<Vec<Swap>> {
    let operation_results = extract_transaction_results(transaction_result);
    let potential_swaps: Vec<Swap> = operation_results.iter().filter_map(build_swap).collect();
    (!potential_swaps.is_empty()).then_some(potential_swaps)
}

fn build_swap(operation_result: &OperationResultTr) -> Option<Swap> {
    match operation_result {
        OperationResultTr::ManageSellOffer(ManageSellOfferResult::Success(offer_result))
        | OperationResultTr::CreatePassiveSellOffer(ManageSellOfferResult::Success(offer_result))
        | OperationResultTr::ManageBuyOffer(ManageBuyOfferResult::Success(offer_result)) => {
            do_build_swap(&offer_result.offer)
        }
        _ => None,
    }
}

fn do_build_swap(offer: &ManageOfferSuccessResultOffer) -> Option<Swap> {
    match offer {
        ManageOfferSuccessResultOffer::Created(offer_entry)
        | ManageOfferSuccessResultOffer::Updated(offer_entry) => {
            if is_stablecoin(&offer_entry.selling) {
                Some(Swap {
                    stablecoin: format_asset(&offer_entry.selling),
                    stablecoin_amount: offer_entry.amount,
                    floating_asset: format_asset(&offer_entry.buying),
                    price_numerator: offer_entry.price.n,
                    price_denominator: offer_entry.price.d,
                })
            } else if is_stablecoin(&offer_entry.buying) {
                Some(Swap {
                    stablecoin: format_asset(&offer_entry.buying),
                    stablecoin_amount: offer_entry.amount * offer_entry.price.n as i64
                        / offer_entry.price.d as i64,
                    floating_asset: format_asset(&offer_entry.selling),
                    price_numerator: offer_entry.price.d,
                    price_denominator: offer_entry.price.n,
                })
            } else {
                None
            }
        }
        ManageOfferSuccessResultOffer::Deleted => None,
    }
}

fn is_successful(result_meta: &TransactionResultMeta) -> bool {
    ALLOW_UNSUCCESSFUL_TRANSACTIONS
        || matches!(
            result_meta.result.result.result,
            TransactionResultResult::TxSuccess(_)
        )
}
