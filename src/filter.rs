use zephyr_sdk::soroban_sdk::xdr::{
    ClaimAtom, ManageBuyOfferResult, ManageOfferSuccessResultOffer, ManageSellOfferResult,
    OperationBody, OperationResultTr, PathPaymentStrictReceiveResult,
    PathPaymentStrictReceiveResultSuccess, PathPaymentStrictSendResult,
    PathPaymentStrictSendResultSuccess, SimplePaymentResult, TransactionEnvelope,
    TransactionResultMeta, TransactionResultResult,
};

use crate::db::swap::Swap;
use crate::utils::{extract_transaction_results, is_stablecoin};

pub fn tmp_path_payments(envelopes: &[TransactionEnvelope]) -> Vec<&TransactionEnvelope> {
    envelopes
        .iter()
        .filter(|transaction| is_path_payment(transaction))
        .collect()
}

pub fn tmp_pps_with_offer_results<'a>(
    transactions: &'a [TransactionEnvelope],
    results: &'a [TransactionResultMeta],
) -> Vec<(&'a TransactionEnvelope, &'a TransactionResultMeta)> {
    transactions
        .iter()
        .zip(results)
        .filter(|(_, result)| is_successful(result))
        .filter_map(|(transaction, result)| tmp_maybe_pp_with_offer_result(transaction, result))
        .collect()
}

fn tmp_maybe_pp_with_offer_result<'a>(
    transaction: &'a TransactionEnvelope,
    result: &'a TransactionResultMeta,
) -> Option<(&'a TransactionEnvelope, &'a TransactionResultMeta)> {
    if is_path_payment(transaction) {
        let results = extract_transaction_results(result);
        let has_offer_result = results.iter().any(|result| {
            !matches!(
                result,
                OperationResultTr::PathPaymentStrictReceive(_)
                    | OperationResultTr::PathPaymentStrictSend(_)
            ) && matches!(
                result,
                OperationResultTr::ManageBuyOffer(_)
                    | OperationResultTr::ManageSellOffer(_)
                    | OperationResultTr::CreatePassiveSellOffer(_)
            )
        });

        if has_offer_result {
            Some((transaction, result))
        } else {
            None
        }
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn swaps_from_path_payment_results(
    transactions: &[TransactionEnvelope],
    results: &[TransactionResultMeta],
) -> Vec<Swap> {
    results
        .iter()
        .filter(|transaction_result| is_successful(transaction_result))
        .zip(transactions)
        .filter_map(build_swaps_from_path_payment_results)
        .flatten()
        .collect::<Vec<Swap>>()
}

fn build_swaps_from_path_payment_results(
    (transaction_result, transaction): (&TransactionResultMeta, &TransactionEnvelope),
) -> Option<Vec<Swap>> {
    if !is_path_payment(transaction) {
        return None;
    }

    build_transaction_ppr_swaps(transaction_result)
}

fn build_transaction_ppr_swaps(transaction_result: &TransactionResultMeta) -> Option<Vec<Swap>> {
    let operation_results = extract_transaction_results(transaction_result);

    let potential_swaps: Vec<Swap> = operation_results
        .iter()
        .filter_map(build_operation_ppr_swaps)
        .flatten()
        .collect();

    (!potential_swaps.is_empty()).then_some(potential_swaps)
}

fn build_operation_ppr_swaps(operation_result: &OperationResultTr) -> Option<Vec<Swap>> {
    match operation_result {
        OperationResultTr::PathPaymentStrictReceive(PathPaymentStrictReceiveResult::Success(
            PathPaymentStrictReceiveResultSuccess { offers, last },
        ))
        | OperationResultTr::PathPaymentStrictSend(PathPaymentStrictSendResult::Success(
            PathPaymentStrictSendResultSuccess { offers, last },
        )) => stub_build_swaps_from_offers_and_last(offers.as_vec(), last),
        _ => None,
    }
}
fn stub_build_swaps_from_offers_and_last(
    offers: &[ClaimAtom],
    _last: &SimplePaymentResult,
) -> Option<Vec<Swap>> {
    offers.iter().map(|_| Default::default()).collect()
}

pub fn swaps_from_path_payment_offers(
    transactions: &[TransactionEnvelope],
    results: &[TransactionResultMeta],
) -> Vec<Swap> {
    results
        .iter()
        .filter(|transaction_result| is_successful(transaction_result))
        .zip(transactions)
        .filter_map(build_swaps_from_path_payment_offers)
        .flatten()
        .collect::<Vec<Swap>>()
}

fn build_swaps_from_path_payment_offers(
    (transaction_result, transaction): (&TransactionResultMeta, &TransactionEnvelope),
) -> Option<Vec<Swap>> {
    if !is_path_payment(transaction) {
        return None;
    }

    build_swaps(transaction_result)
}

pub fn swaps_from_elsewhere(
    transactions: &[TransactionEnvelope],
    results: &[TransactionResultMeta],
) -> Vec<Swap> {
    results
        .iter()
        .filter(|transaction_result| is_successful(transaction_result))
        .zip(transactions)
        .filter_map(build_swaps_from_elsewhere)
        .flatten()
        .collect::<Vec<Swap>>()
}

fn build_swaps_from_elsewhere(
    (transaction_result, transaction): (&TransactionResultMeta, &TransactionEnvelope),
) -> Option<Vec<Swap>> {
    if is_path_payment(transaction) {
        return None;
    }

    build_swaps(transaction_result)
}

fn is_path_payment(transaction: &TransactionEnvelope) -> bool {
    let operations = crate::utils::extract_transaction_operations(transaction);

    operations.iter().any(|operation| {
        matches!(
            operation,
            OperationBody::PathPaymentStrictReceive(_) | OperationBody::PathPaymentStrictSend(_)
        )
    })
}

#[allow(dead_code)]
pub fn swaps(transaction_results: &[TransactionResultMeta]) -> Vec<Swap> {
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
