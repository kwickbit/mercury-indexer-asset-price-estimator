use zephyr_sdk::{
    soroban_sdk::xdr::{
        AlphaNum12, AlphaNum4, Asset, Operation, OperationBody, PathPaymentStrictReceiveOp,
        PathPaymentStrictSendOp, PaymentOp, TransactionEnvelope, TransactionResultMeta,
        TransactionResultResult, VecM,
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
                "Sequence {} has {} successful {} transactions:\n{}",
                sequence,
                successful.len(),
                ASSET,
                log_usdc_transactions(&successful),
            ),
            None,
        );
    }
}

fn log_usdc_transactions(transactions: &[&TransactionEnvelope]) -> String {
    transactions.iter().map(format_usdc_transaction).collect()
}

fn format_usdc_transaction(transaction: &&TransactionEnvelope) -> String {
    let mut result = String::new();
    let operations = transaction_operations(transaction);

    for operation in operations.iter() {
        result.push_str("Operation #: ");
        result.push_str(&fmt_op_in_usdc_txn(&operation.body));
        result.push('\n');
    }

    result
}

fn fmt_op_in_usdc_txn(operation: &OperationBody) -> String {
    match &operation {
        OperationBody::Payment(PaymentOp {
            asset,
            amount,
            destination,
        }) => format!(
            "Payment of {amount} {} to {destination}",
            format_asset(asset)
        ),
        OperationBody::PathPaymentStrictReceive(PathPaymentStrictReceiveOp {
            send_asset,
            send_max,
            destination,
            dest_asset,
            dest_amount,
            path,
        }) => format!(
            "Path payment (receive) to {destination}:\nmaximum send of {send_max} {} to {dest_amount} {}.\nPath: {}",
            format_asset(send_asset), format_asset(dest_asset), format_path(path)
        ),
        OperationBody::PathPaymentStrictSend(PathPaymentStrictSendOp {
            send_asset,
            send_amount,
            destination,
            dest_asset,
            dest_min,
            path,
        }) => format!(
            "Path payment (send) to {destination}:\n{send_amount} {} to minimum of {dest_min} {}.\nPath: {}",
            format_asset(send_asset), format_asset(dest_asset), format_path(path)
        ),
        _ => unreachable!("This case should never occur due to prior filtering"),
    }
}

fn format_path(path: &VecM<Asset, 5>) -> String {
    path.iter().map(format_asset).collect::<Vec<&str>>().join(" => ")
}

fn format_asset(asset: &Asset) -> &str {
    match asset {
        Asset::Native => "XLM",
        Asset::CreditAlphanum4(AlphaNum4 { asset_code, .. }) => asset_to_str(asset_code.as_slice()),
        Asset::CreditAlphanum12(AlphaNum12 { asset_code, .. }) => {
            asset_to_str(asset_code.as_slice())
        }
    }
}

fn asset_to_str(code: &[u8]) -> &str {
    std::str::from_utf8(code).expect("Invalid asset code")
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

fn is_usdc(transaction: &TransactionEnvelope) -> bool {
    let operations = transaction_operations(transaction);

    if operations.is_empty() {
        return false;
    }

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

fn transaction_operations(transaction: &TransactionEnvelope) -> VecM<Operation, 100> {
    match transaction {
        TransactionEnvelope::TxV0(envelope) => envelope.tx.operations.clone(),
        TransactionEnvelope::Tx(envelope) => envelope.tx.operations.clone(),
        _ => Default::default(),
    }
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
