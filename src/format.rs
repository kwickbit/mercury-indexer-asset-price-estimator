use num_format::{Locale, ToFormattedString};

use zephyr_sdk::soroban_sdk::xdr::{
    Asset, ClaimAtom, ClaimLiquidityAtom, ClaimOfferAtom, ClaimOfferAtomV0, OperationBody,
    OperationResult, OperationResultTr, PathPaymentStrictReceiveOp, PathPaymentStrictReceiveResult,
    PathPaymentStrictReceiveResultSuccess, PathPaymentStrictSendOp, PathPaymentStrictSendResult,
    PathPaymentStrictSendResultSuccess, TransactionEnvelope, TransactionResultMeta,
    TransactionResultResult, VecM,
};

use crate::transaction::InterestingTransaction;
use crate::utils::format_asset;

pub fn format_interesting_transaction(
    sequence: u32,
    transaction: &InterestingTransaction,
    transaction_number: usize,
    op_formatter: fn(&OperationBody, &OperationResult) -> String,
) -> String {
    let mut result = String::new();

    result.push_str(&format!(
        "Sequence {}, transaction #{}, ",
        sequence, transaction_number
    ));

    transaction
        .operations
        .iter()
        .zip(transaction.results.iter())
        .enumerate()
        .for_each(|(op_index, (operation, op_result))| {
            result.push_str(&format!("operation #{}: ", op_index + 1));
            result.push_str(&op_formatter(
                &operation.body,
                op_result,
            ));
        });

    result
}

pub fn format_path_payment(
    operation: &OperationBody,
    result: &OperationResult,
) -> String {
    match operation {
        OperationBody::PathPaymentStrictReceive(PathPaymentStrictReceiveOp {
            send_asset,
            send_max,
            dest_asset,
            dest_amount,
            path,
            ..
        }) => format!(
            "Path payment (receive): maximum send of {} {} to {} {}. Path: {}",
            format_amount(send_max),
            format_asset(send_asset),
            format_amount(dest_amount),
            format_asset(dest_asset),
            format_path(send_asset, path, dest_asset, result),
        ),
        OperationBody::PathPaymentStrictSend(PathPaymentStrictSendOp {
            send_asset,
            send_amount,
            dest_asset,
            dest_min,
            path,
            ..
        }) => format!(
            "Path payment (send): {} {} to minimum of {} {}. Path: {}",
            format_amount(send_amount),
            format_asset(send_asset),
            format_amount(dest_min),
            format_asset(dest_asset),
            format_path(send_asset, path, dest_asset, result),
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

fn format_path(
    send_asset: &Asset,
    path: &VecM<Asset, 5>,
    dest_asset: &Asset,
    op_result: &OperationResult,
) -> String {
    let mut result: String = format!(
        "{} => {} => {}",
        format_asset(send_asset),
        path.iter()
            .map(format_asset)
            .collect::<Vec<String>>()
            .join(" => "),
        format_asset(dest_asset)
    );

    // We pick the offers that fulfilled the path payment, if present
    match op_result {
        OperationResult::OpInner(OperationResultTr::PathPaymentStrictReceive(
            PathPaymentStrictReceiveResult::Success(PathPaymentStrictReceiveResultSuccess {
                offers,
                ..
            }),
        ))
        | OperationResult::OpInner(OperationResultTr::PathPaymentStrictSend(
            PathPaymentStrictSendResult::Success(PathPaymentStrictSendResultSuccess {
                offers, ..
            }),
        )) => {
            result.push_str(" // ");

            offers
                .to_vec()
                .iter()
                .for_each(|claim_atom| match claim_atom {
                    ClaimAtom::V0(ClaimOfferAtomV0 {
                        amount_sold,
                        asset_sold,
                        amount_bought,
                        asset_bought,
                        ..
                    })
                    | ClaimAtom::OrderBook(ClaimOfferAtom {
                        amount_sold,
                        asset_sold,
                        amount_bought,
                        asset_bought,
                        ..
                    })
                    | ClaimAtom::LiquidityPool(ClaimLiquidityAtom {
                        amount_sold,
                        asset_sold,
                        amount_bought,
                        asset_bought,
                        ..
                    }) => result.push_str(&format!(
                        "-- Sold {} {} for {} {}",
                        format_amount(amount_sold),
                        format_asset(asset_sold),
                        format_amount(amount_bought),
                        format_asset(asset_bought)
                    )),
                });

            result
        }
        _ => {
            result.push_str(&format!(" // {:?}", op_result));
            result
        }
    }
}

#[allow(dead_code)]
fn foo(
    events: Vec<(&TransactionEnvelope, &TransactionResultMeta)>,
    logger: impl Fn(&str),
    sequence: u32,
) {
    // We count how many transactions are successful
    let success_count = events
        .iter()
        .filter(|(_, result)| {
            matches!(
                result.result.result.result,
                TransactionResultResult::TxSuccess(_)
            )
        })
        .count();

    // We count path payments
    let path_payments: Vec<(&TransactionEnvelope, TransactionResultResult)> = events
        .iter()
        .filter_map(|(transaction, result)| {
            let ops = crate::utils::extract_transaction_operations(transaction);
            if ops.iter().any(|op| {
                matches!(op.body, OperationBody::PathPaymentStrictReceive(_))
                    || matches!(op.body, OperationBody::PathPaymentStrictSend(_))
            }) {
                Some((*transaction, result.result.result.result.clone()))
            } else {
                None
            }
        })
        .collect();

    // let successful_path_payments = "no";
    let successful_path_payments = path_payments
        .iter()
        .filter(|(_, result)| matches!(result, TransactionResultResult::TxSuccess(_)))
        .count();

    logger(
        &format!(
            "Sequence {sequence} with {} transactions, {success_count} of them successful; {} of the total are path payments, of which {successful_path_payments} succeed",
            events.len(),
            path_payments.len()
        )
    );
}
