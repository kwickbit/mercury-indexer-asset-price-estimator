use zephyr_sdk::soroban_sdk::xdr::{
    CreatePassiveSellOfferOp, ManageBuyOfferOp, ManageBuyOfferResult, ManageSellOfferOp,
    ManageSellOfferResult, Operation, OperationBody, OperationResultTr, PathPaymentStrictReceiveOp,
    PathPaymentStrictSendOp, TransactionEnvelope, TransactionResultMeta, TransactionResultResult,
};

use crate::config::ALLOW_UNSUCCESSFUL_TRANSACTIONS;
use crate::transaction::InterestingTransaction;
use crate::utils::{asset_matches, ASSET};

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
        }) => asset_matches(selling, ASSET) || asset_matches(buying, ASSET),
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
            asset_matches(send_asset, ASSET)
                || asset_matches(dest_asset, ASSET)
                || path.iter().any(|asset| asset_matches(asset, ASSET))
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
