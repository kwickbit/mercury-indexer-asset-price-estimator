use zephyr_sdk::soroban_sdk::xdr::{
    AlphaNum4, Asset, OperationBody, PathPaymentStrictReceiveOp, PathPaymentStrictSendOp,
    TransactionEnvelope, TransactionResultMeta,
};

use crate::transaction::InterestingTransaction;
use crate::utils::ASSET;

pub fn interesting_transactions<'a>(
    events: &[(&'a TransactionEnvelope, &'a TransactionResultMeta)],
) -> Vec<InterestingTransaction<'a>> {
    events
        .iter()
        .filter_map(|(envelope, result_meta)| {
            if is_usdc(envelope) {
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
