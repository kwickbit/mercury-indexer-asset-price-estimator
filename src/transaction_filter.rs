use zephyr_sdk::soroban_sdk::xdr::{
    Asset, ClaimAtom, ManageBuyOfferResult, ManageSellOfferResult, Operation, OperationBody, OperationResultTr, PathPaymentStrictReceiveOp, PathPaymentStrictSendOp, TransactionEnvelope, TransactionResultMeta, TransactionResultResult
};

use crate::transaction::InterestingTransaction;
use crate::utils::{asset_matches, extract_transaction_results, ASSET};

#[allow(dead_code)]
pub fn get_claim_atoms<'a>(
    events: &[(&'a TransactionEnvelope, &'a TransactionResultMeta)],
) -> Vec<ClaimAtom> {
    let claim_atoms = Vec::new();

    let _op_results = {
        events
            .iter()
            .flat_map(|(_, result_meta)| extract_transaction_results(result_meta))
            .collect::<Vec<OperationResultTr>>()
    };

    // op_results
    //     .iter()
    //     .filter_map(|op_result| {
    //         match op_result {
    //             OperationResultTr::PathPaymentStrictReceive(success) => {
    //                 match success {
    //                     PathPaymentStrictReceiveResult::Success => {
    //                         claim_atoms.push(ClaimAtom::PathPaymentStrictReceive(
    //                             PathPaymentStrictResult::Success {
    //                                 amount_sold,
    //                                 asset_sold,
    //                                 amount_bought,
    //                                 asset_bought,
    //                             },
    //                         ));
    //                     }
    //                     _ => {}
    //                 }
    //                 Some(ClaimAtom::PathPaymentStrictReceive(PathPaymentStrictResult::Success))
    //             }
    //             _ => None,
    //         }
    //         });


    claim_atoms
}

#[allow(dead_code)]
pub fn interesting_transactions<'a>(
    events: &[(&'a TransactionEnvelope, &'a TransactionResultMeta)],
) -> Vec<InterestingTransaction> {
    events
        .iter()
        .filter_map(|(envelope, result_meta)| {
            if is_successful(result_meta) && is_usdc_path_payment(envelope) {
                Some(InterestingTransaction::new(envelope, result_meta))
            } else {
                None
            }
        })
        .collect()
}

#[allow(dead_code)]
fn is_offer(result_meta: &TransactionResultMeta) -> bool {
    let op_results = crate::utils::extract_transaction_results(result_meta);

    op_results.iter().any(is_offer_op)
}

fn is_offer_op(op_result: &OperationResultTr) -> bool {
    matches!(
        op_result,
        OperationResultTr::ManageSellOffer(ManageSellOfferResult::Success(_))
            | OperationResultTr::ManageBuyOffer(ManageBuyOfferResult::Success(_))
            | OperationResultTr::CreatePassiveSellOffer(ManageSellOfferResult::Success(_))
    )
}

#[allow(dead_code)]
fn is_usdc_path_payment(transaction: &TransactionEnvelope) -> bool {
    let operations = crate::utils::extract_transaction_operations(transaction);

    if operations.is_empty() {
        return false;
    }

    operations.iter().any(is_usdc_path_payment_op)
}

fn is_usdc_path_payment_op(op: &Operation) -> bool {
    match &op.body {
        OperationBody::PathPaymentStrictReceive(PathPaymentStrictReceiveOp {
            send_asset: Asset::CreditAlphanum4(send_asset),
            dest_asset: Asset::CreditAlphanum4(dest_asset),
            path,
            ..
        }) => {
            asset_matches(send_asset, ASSET)
                || asset_matches(dest_asset, ASSET)
                || path.iter().any(|asset| match asset {
                    Asset::CreditAlphanum4(inner_asset) => asset_matches(inner_asset, ASSET),
                    _ => false,
                })
        }
        OperationBody::PathPaymentStrictSend(PathPaymentStrictSendOp {
            send_asset: Asset::CreditAlphanum4(send_asset),
            dest_asset: Asset::CreditAlphanum4(dest_asset),
            path,
            ..
        }) => {
            asset_matches(send_asset, ASSET)
                || asset_matches(dest_asset, ASSET)
                || path.iter().any(|asset| match asset {
                    Asset::CreditAlphanum4(inner_asset) => asset_matches(inner_asset, ASSET),
                    _ => false,
                })
        }
        _ => false,
    }
}

fn is_successful(result_meta: &TransactionResultMeta) -> bool {
    matches!(
        result_meta.result.result.result,
        TransactionResultResult::TxSuccess(_)
    )
}
