use zephyr_sdk::soroban_sdk::xdr::{
    CreatePassiveSellOfferOp, ManageBuyOfferOp, ManageBuyOfferResult,
    ManageOfferSuccessResultOffer, ManageSellOfferOp, ManageSellOfferResult, Operation,
    OperationBody, OperationResultTr, PathPaymentStrictReceiveOp, PathPaymentStrictSendOp,
    TransactionEnvelope, TransactionResultMeta, TransactionResultResult,
};

use crate::config::ALLOW_UNSUCCESSFUL_TRANSACTIONS;
use crate::swap::Swap;
use crate::transaction::InterestingTransaction;
use crate::utils::{extract_transaction_results, format_asset, is_stablecoin};

pub fn swaps(transaction_results: Vec<TransactionResultMeta>) -> Vec<Swap> {
    transaction_results
        .iter()
        .filter(|transaction_result| is_successful(&transaction_result))
        .filter_map(|transaction_result| build_swaps(transaction_result))
        .flatten()
        .collect()
}

fn build_swaps(transaction_result: &TransactionResultMeta) -> Option<Vec<Swap>> {
    let operation_results = extract_transaction_results(transaction_result);

    let potential_swaps: Vec<Option<Swap>> = operation_results
        .iter()
        .map(|operation_result| build_swap(operation_result))
        .collect();

    if potential_swaps.is_empty() {
        None
    } else {
        Some(
            potential_swaps
                .into_iter()
                .filter_map(|swap| swap)
                .collect(),
        )
    }
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

#[allow(dead_code)]
pub fn interesting_transactions(
    events: &[(&TransactionEnvelope, &TransactionResultMeta)],
    op_filter: Option<fn(&TransactionEnvelope) -> bool>,
    result_filter: Option<fn(&TransactionResultMeta) -> bool>,
) -> Vec<InterestingTransaction> {
    let op_filter = op_filter.unwrap_or(|_| true);
    let result_filter = result_filter.unwrap_or(|_| true);

    events
        .iter()
        .filter_map(|(envelope, result_meta)| {
            if is_successful(result_meta) && op_filter(envelope) && result_filter(result_meta) {
                Some(InterestingTransaction::new(envelope, result_meta))
            } else {
                None
            }
        })
        .collect()
}

pub fn is_usdc_op(transaction: &TransactionEnvelope) -> bool {
    is_usdc_path_payment(transaction) || is_usdc_offer(transaction)
}

#[allow(dead_code)]
pub fn is_offer_result(result_meta: &TransactionResultMeta) -> bool {
    let op_results = crate::utils::extract_transaction_results(result_meta);

    op_results.iter().any(is_offer_op_result)
}

fn is_offer_op_result(op_result: &OperationResultTr) -> bool {
    matches!(
        op_result,
        OperationResultTr::ManageSellOffer(ManageSellOfferResult::Success(_))
            | OperationResultTr::ManageBuyOffer(ManageBuyOfferResult::Success(_))
            | OperationResultTr::CreatePassiveSellOffer(ManageSellOfferResult::Success(_))
    )
}

fn is_usdc_offer(transaction: &TransactionEnvelope) -> bool {
    let operations = crate::utils::extract_transaction_operations(transaction);

    if operations.is_empty() {
        return false;
    }

    operations.iter().any(is_usdc_offer_op)
}

fn is_usdc_offer_op(op: &Operation) -> bool {
    match &op.body {
        OperationBody::ManageSellOffer(ManageSellOfferOp {
            selling, buying, ..
        })
        | OperationBody::ManageBuyOffer(ManageBuyOfferOp {
            selling, buying, ..
        })
        | OperationBody::CreatePassiveSellOffer(CreatePassiveSellOfferOp {
            selling, buying, ..
        }) => is_stablecoin(selling) || is_stablecoin(buying),
        _ => false,
    }
}

#[allow(dead_code)]
pub fn is_usdc_path_payment(transaction: &TransactionEnvelope) -> bool {
    let operations = crate::utils::extract_transaction_operations(transaction);

    if operations.is_empty() {
        return false;
    }

    operations.iter().any(is_usdc_path_payment_op)
}

fn is_usdc_path_payment_op(op: &Operation) -> bool {
    match &op.body {
        OperationBody::PathPaymentStrictReceive(PathPaymentStrictReceiveOp {
            send_asset,
            dest_asset,
            path,
            ..
        })
        | OperationBody::PathPaymentStrictSend(PathPaymentStrictSendOp {
            send_asset,
            dest_asset,
            path,
            ..
        }) => {
            is_stablecoin(send_asset)
                || is_stablecoin(dest_asset)
                || path.iter().any(|asset| is_stablecoin(asset))
        }
        _ => false,
    }
}

fn is_successful(result_meta: &TransactionResultMeta) -> bool {
    ALLOW_UNSUCCESSFUL_TRANSACTIONS
        || matches!(
            result_meta.result.result.result,
            TransactionResultResult::TxSuccess(_)
        )
}
