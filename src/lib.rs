use zephyr_sdk::{
    soroban_sdk::xdr::{
        AlphaNum4, Asset, OperationBody, PathPaymentStrictReceiveOp, PathPaymentStrictSendOp,
        PaymentOp, TransactionEnvelope, TransactionResultMeta, TransactionResultResult,
    },
    EnvClient,
};

const ASSET: &str = "USDC";

#[no_mangle]
pub extern "C" fn on_close() {
    let env = EnvClient::new();
    let reader = env.reader();
    let sequence = reader.ledger_sequence();
    let events = reader.envelopes_with_meta();
    let successful = successful_usdc_txns(&events);

    if successful.is_empty() && sequence % 12 == 0 {
        env.log().debug(&format!("Sequence {}", sequence), None);
    }

    if !successful.is_empty() {
        env.log().debug(
            &format!(
                "Sequence {} has {} successful {} transactions",
                sequence,
                successful.len(),
                ASSET,
            ),
            None,
        );
    }
}

fn successful_usdc_txns<'a>(
    events: &[(&'a TransactionEnvelope, &TransactionResultMeta)],
) -> Vec<&'a TransactionEnvelope> {
    events.iter().filter_map(is_successful_usdc_txn).collect()
}

fn is_successful_usdc_txn<'a>(
    (envelope, result_meta): &(&'a TransactionEnvelope, &TransactionResultMeta),
) -> Option<&'a TransactionEnvelope> {
    if is_usdc(envelope) && is_successful(result_meta) {
        Some(envelope)
    } else {
        None
    }
}

fn is_usdc(envelope: &TransactionEnvelope) -> bool {
    let operations = match envelope {
        TransactionEnvelope::TxV0(v0env) => &v0env.tx.operations,
        TransactionEnvelope::Tx(v0env) => &v0env.tx.operations,
        _ => return false,
    };

    operations.iter().any(|op| match &op.body {
        OperationBody::Payment(PaymentOp {
            asset: Asset::CreditAlphanum4(asset),
            ..
        }) => asset_matches(asset, ASSET),
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

fn is_successful(result_meta: &TransactionResultMeta) -> bool {
    matches!(
        result_meta.result.result.result,
        TransactionResultResult::TxSuccess(_)
    )
}

fn asset_matches(asset: &AlphaNum4, code: &str) -> bool {
    asset.asset_code.as_slice() == code.as_bytes()
}
