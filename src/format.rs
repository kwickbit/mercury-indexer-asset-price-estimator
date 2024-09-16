use num_format::{Locale, ToFormattedString};

use zephyr_sdk::soroban_sdk::xdr::{
    Asset, ClaimAtom, ClaimLiquidityAtom, ClaimOfferAtom, ClaimOfferAtomV0, ManageBuyOfferResult,
    ManageOfferSuccessResultOffer, ManageSellOfferResult, OperationBody, OperationResult,
    OperationResultTr, PathPaymentStrictReceiveOp, PathPaymentStrictReceiveResult,
    PathPaymentStrictReceiveResultSuccess, PathPaymentStrictSendOp, PathPaymentStrictSendResult,
    PathPaymentStrictSendResultSuccess, TransactionEnvelope, TransactionResultMeta,
    TransactionResultResult, VecM,
};

use crate::transaction::InterestingTransaction;
use crate::utils::format_asset;

#[allow(dead_code)]
pub fn format_interesting_transaction(
    sequence: u32,
    transaction: &InterestingTransaction,
    transaction_number: usize,
    op_formatter: fn(&OperationBody, &OperationResult) -> String,
) -> String {
    let mut result = String::new();

    result.push_str(&format!(
        "Sequence {}, transaction #{}, hash {}",
        sequence, transaction_number, transaction.hash
    ));

    transaction
        .operations
        .iter()
        .zip(transaction.results.iter())
        .enumerate()
        .for_each(|(op_index, (operation, op_result))| {
            result.push_str(&format!(" // operation #{}: ", op_index + 1));
            result.push_str(&op_formatter(&operation.body, op_result));
        });

    result
}

#[allow(dead_code)]
pub fn empty_formatter(_operation: &OperationBody, _op_result: &OperationResult) -> String {
    "".to_string()
}

#[allow(dead_code)]
pub fn format_offer(_operation: &OperationBody, op_result: &OperationResult) -> String {
    let mut result = "Offer result: ".to_string();

    let formatted_op_result = match op_result {
        OperationResult::OpInner(result_tr) => match result_tr {
            OperationResultTr::ManageSellOffer(ManageSellOfferResult::Success(success_result))
            | OperationResultTr::ManageBuyOffer(ManageBuyOfferResult::Success(success_result))
            | OperationResultTr::CreatePassiveSellOffer(ManageSellOfferResult::Success(
                success_result,
            )) => &success_result
                .offers_claimed
                .iter()
                .map(format_claim_atom)
                .collect::<Vec<String>>()
                .join(" - "),
            _ => "Unreachable non-Offer result",
        },
        _ => "Unreachable OpResult failure",
    };

    result.push_str(formatted_op_result);

    result
}

#[allow(dead_code)]
pub fn format_path_payment(operation: &OperationBody, op_result: &OperationResult) -> String {
    let mut result_str = "Path payment ".to_string();

    match operation {
        OperationBody::PathPaymentStrictReceive(PathPaymentStrictReceiveOp {
            send_asset,
            send_max,
            dest_asset,
            dest_amount,
            path,
            ..
        }) => result_str.push_str(&format!(
            "(receive): maximum send of {} {} to {} {}. Path: {}",
            format_amount(send_max),
            format_asset(send_asset),
            format_amount(dest_amount),
            format_asset(dest_asset),
            format_path(send_asset, path, dest_asset, op_result)
        )),
        OperationBody::PathPaymentStrictSend(PathPaymentStrictSendOp {
            send_asset,
            send_amount,
            dest_asset,
            dest_min,
            path,
            ..
        }) => result_str.push_str(&format!(
            "(send): {} {} to minimum of {} {}. Path: {}",
            format_amount(send_amount),
            format_asset(send_asset),
            format_amount(dest_min),
            format_asset(dest_asset),
            format_path(send_asset, path, dest_asset, op_result)
        )),
        _ => unreachable!("This case should never occur due to prior filtering"),
    }

    match op_result {
        OperationResult::OpInner(OperationResultTr::PathPaymentStrictReceive(
            PathPaymentStrictReceiveResult::Success(PathPaymentStrictReceiveResultSuccess {
                offers,
                last,
            }),
        ))
        | OperationResult::OpInner(OperationResultTr::PathPaymentStrictSend(
            PathPaymentStrictSendResult::Success(PathPaymentStrictSendResultSuccess {
                offers,
                last,
            }),
        )) => result_str.push_str(&format!(
            " // path payment result!! {} offers, last {:?}",
            offers.len(),
            last
        )),
        OperationResult::OpInner(OperationResultTr::ManageBuyOffer(
            ManageBuyOfferResult::Success(offer_result),
        ))
        | OperationResult::OpInner(OperationResultTr::ManageSellOffer(
            ManageSellOfferResult::Success(offer_result),
        )) => {
            result_str.push_str(&format!(
                " // offer result: {}, with {} offers claimed.",
                format_offer_result(&offer_result.offer),
                offer_result.offers_claimed.len()
            ));
        }
        _ => {}
    }

    result_str
}

fn format_amount(amount: &i64) -> String {
    let conversion_factor = 10_000_000;
    let integer_amount = *amount / conversion_factor;
    let formatted_integer = integer_amount.to_formatted_string(&Locale::en);
    let fractional_amount = (*amount % conversion_factor) as f64;
    format!("{}.{}", formatted_integer, fractional_amount)
}

fn format_offer_result(offer_result: &ManageOfferSuccessResultOffer) -> String {
    let mut result = String::new();

    match offer_result {
        ManageOfferSuccessResultOffer::Created(offer) => result.push_str(&format!(
            "created offer: {} {} for {} {}",
            format_amount(&offer.amount),
            format_asset(&offer.selling),
            format_amount(&(offer.price.n as i64 / offer.price.d as i64)),
            format_asset(&offer.buying)
        )),
        ManageOfferSuccessResultOffer::Updated(offer) => result.push_str(&format!(
            "updated offer: {} {} for {} {}",
            format_amount(&offer.amount),
            format_asset(&offer.selling),
            format_amount(&(offer.price.n as i64 / offer.price.d as i64)),
            format_asset(&offer.buying)
        )),
        ManageOfferSuccessResultOffer::Deleted => result.push_str("deleted offer"),
    }

    result
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
            result.push_str(" // path payment: ");

            offers
                .to_vec()
                .iter()
                .for_each(|claim_atom| result.push_str(&format_claim_atom(claim_atom)));
        }
        OperationResult::OpInner(OperationResultTr::ManageBuyOffer(
            ManageBuyOfferResult::Success(offer_result),
        ))
        | OperationResult::OpInner(OperationResultTr::ManageSellOffer(
            ManageSellOfferResult::Success(offer_result),
        )) => {
            result.push_str(&format!(
                " // offer result: {}, with {} offers claimed.",
                format_offer_result(&offer_result.offer),
                offer_result.offers_claimed.len()
            ));
        }
        _ => {
            result.push_str(&format!(" // {:?}", op_result));
        }
    }

    result
}

pub fn format_claim_atom(claim_atom: &ClaimAtom) -> String {
    match claim_atom {
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
        }) => format!(
            "-- Sold {} {} for {} {}",
            format_amount(amount_sold),
            format_asset(asset_sold),
            format_amount(amount_bought),
            format_asset(asset_bought)
        ),
    }
}

#[allow(dead_code)]
fn report_transactions_by_success_and_path_payment(
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
