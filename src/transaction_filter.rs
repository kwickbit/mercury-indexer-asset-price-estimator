use zephyr_sdk::soroban_sdk::xdr::{
    Asset, OperationBody, PathPaymentStrictReceiveOp, PathPaymentStrictSendOp, TransactionEnvelope,
    TransactionResultMeta, TransactionResultResult,
};

use crate::transaction::InterestingTransaction;
use crate::utils::{asset_matches, ASSET};

pub fn interesting_transactions<'a>(
    events: &[(&'a TransactionEnvelope, &'a TransactionResultMeta)],
) -> Vec<InterestingTransaction> {
    events
        .iter()
        .filter_map(|(envelope, result_meta)| {
            if is_successful(result_meta) && is_usdc(envelope) {
                Some(InterestingTransaction::new(envelope, result_meta))
            } else {
                None
            }
        })
        .collect()
}

fn is_usdc(transaction: &TransactionEnvelope) -> bool {
    let operations = crate::utils::extract_transaction_operations(transaction);

    if operations.is_empty() {
        return false;
    }

    operations.iter().any(|op| match &op.body {
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
    })
}

fn is_successful(result_meta: &TransactionResultMeta) -> bool {
    matches!(
        result_meta.result.result.result,
        TransactionResultResult::TxSuccess(_)
    )
}
