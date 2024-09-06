use zephyr_sdk::{soroban_sdk::xdr::{AlphaNum12, AlphaNum4, Asset, OperationBody, PathPaymentStrictReceiveOp, PathPaymentStrictSendOp, PaymentOp, TransactionEnvelope, VecM}, EnvLogger};

use crate::utils;

pub fn log(env: &EnvLogger) -> impl Fn(&str) + '_ {
    move |args| env.debug(args, None)
}

pub fn log_usdc_transactions(transactions: &[&TransactionEnvelope]) -> String {
    transactions.iter().map(format_usdc_transaction).collect()
}

fn format_usdc_transaction(transaction: &&TransactionEnvelope) -> String {
    let mut result = String::new();
    let operations = utils::transaction_operations(transaction);

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
    path.iter()
        .map(format_asset)
        .collect::<Vec<&str>>()
        .join(" => ")
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
