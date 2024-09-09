use num_format::{Locale, ToFormattedString};

use zephyr_sdk::soroban_sdk::xdr::{
    AlphaNum12, AlphaNum4, Asset, OperationBody, PathPaymentStrictReceiveOp,
    PathPaymentStrictSendOp, VecM,
};

use crate::transaction::InterestingTransaction;
use crate::utils;

pub fn format_interesting_transactions(transactions: &[InterestingTransaction]) -> String {
    transactions
        .iter()
        .enumerate()
        .map(|(index, transaction)| format_interesting_transaction(transaction, index + 1))
        .collect()
}

fn format_interesting_transaction(
    transaction: &InterestingTransaction,
    sequence_number: usize,
) -> String {
    let mut result = String::new();

    result.push_str(&format!("Transaction #{}:\n", sequence_number));

    for (op_index, operation) in transaction.operations.iter().enumerate() {
        result.push_str(&format!(
            "Operation #{}.{}: ",
            sequence_number,
            op_index + 1
        ));
        result.push_str(&format_operation_in_interesting_transaction(
            &operation.body,
            transaction.is_successful(),
        ));
        result.push('\n');
    }

    result
}

fn format_operation_in_interesting_transaction(
    operation: &OperationBody,
    is_successful: bool,
) -> String {
    match operation {
        OperationBody::PathPaymentStrictReceive(PathPaymentStrictReceiveOp {
            send_asset,
            send_max,
            destination,
            dest_asset,
            dest_amount,
            path,
        }) => format!(
            "Path payment (receive) to {destination}:\nmaximum send of {} {} to {} {}.\nPath: {}\nSuccessful: {is_successful}",
            format_amount(send_max),
            format_asset(send_asset),
            format_amount(dest_amount),
            format_asset(dest_asset),
            format_path(send_asset, path, dest_asset),
        ),
        OperationBody::PathPaymentStrictSend(PathPaymentStrictSendOp {
            send_asset,
            send_amount,
            destination,
            dest_asset,
            dest_min,
            path,
        }) => format!(
            "Path payment (send) to {destination}:\n{} {} to minimum of {} {}.\nPath: {}\nSuccessful: {is_successful}",
            format_amount(send_amount),
            format_asset(send_asset),
            format_amount(dest_min),
            format_asset(dest_asset),
            format_path(send_asset, path, dest_asset),
        ),
        _ => unreachable!("This case should never occur due to prior filtering"),
    }
}

fn format_amount(amount: &i64) -> String {
    let conversion_factor = 10_000_000;
    let integer_amount = *amount / conversion_factor;
    let formatted_integer = integer_amount.to_formatted_string(&Locale::en);
    let fractional_amount = (*amount % conversion_factor) as f64;
    format!("{}.{}", formatted_integer, fractional_amount)
}

fn format_path(send_asset: &Asset, path: &VecM<Asset, 5>, dest_asset: &Asset) -> String {
    format!(
        "{} => {} => {}",
        format_asset(send_asset),
        path.iter()
            .map(format_asset)
            .collect::<Vec<String>>()
            .join(" => "),
        format_asset(dest_asset)
    )
}

fn format_asset(asset: &Asset) -> String {
    match asset {
        Asset::Native => "XLM".to_string(),
        Asset::CreditAlphanum4(AlphaNum4 { asset_code, .. }) => {
            format_nonnative_asset(asset_code.as_slice())
        }
        Asset::CreditAlphanum12(AlphaNum12 { asset_code, .. }) => {
            format_nonnative_asset(asset_code.as_slice())
        }
    }
}

fn format_nonnative_asset(asset_code: &[u8]) -> String {
    utils::bytes_to_string(asset_code)
        .chars()
        .filter(|char| char.is_ascii_alphabetic())
        .collect()
}
