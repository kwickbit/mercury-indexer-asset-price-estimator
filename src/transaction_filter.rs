use zephyr_sdk::soroban_sdk::xdr::{
    AlphaNum4, Asset, OperationBody, PathPaymentStrictReceiveOp, PathPaymentStrictSendOp,
    TransactionEnvelope, TransactionResultMeta, TransactionResultResult,
};

use crate::transaction::InterestingTransaction;
use crate::utils::ASSET;

pub fn interesting_transactions<'a>(
    events: &[(&'a TransactionEnvelope, &'a TransactionResultMeta)],
) -> Vec<InterestingTransaction> {
    events
        .iter()
        .filter_map(|(envelope, result_meta)| {
            if is_usdc(envelope) && is_successful(result_meta) {
                Some(InterestingTransaction::new(envelope, result_meta))
            } else {
                None
            }
        })
        .collect()
}

fn is_successful(result_meta: &TransactionResultMeta) -> bool {
    matches!(
        result_meta.result.result.result,
        TransactionResultResult::TxSuccess(_)
    )
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
            ..
        }) => asset_matches(send_asset, ASSET) || asset_matches(dest_asset, ASSET),
        OperationBody::PathPaymentStrictSend(PathPaymentStrictSendOp {
            send_asset: Asset::CreditAlphanum4(send_asset),
            dest_asset: Asset::CreditAlphanum4(dest_asset),
            ..
        }) => asset_matches(send_asset, ASSET) || asset_matches(dest_asset, ASSET),
        _ => false,
    })
}

fn asset_matches(asset: &AlphaNum4, code: &str) -> bool {
    asset.asset_code.as_slice() == code.as_bytes()
}
