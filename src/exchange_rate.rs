use zephyr_sdk::soroban_sdk::xdr::{
    Asset, ClaimAtom, Operation, OperationBody, OperationResult, OperationResultTr,
    PathPaymentStrictReceiveResult, PathPaymentStrictSendResult, SimplePaymentResult, VecM,
};

use crate::utils;


#[allow(dead_code)]
pub struct ExchangeRate {
    pub asset: String,
    pub usd_value: f64,
}

#[allow(dead_code)]
pub fn extract_exchange_rates(
    operation: &Operation,
    result: &OperationResult,
) -> Option<Vec<ExchangeRate>> {
    if let OperationResult::OpInner(inner) = result {
        match (&operation.body, inner) {
            (
                OperationBody::PathPaymentStrictReceive(_),
                OperationResultTr::PathPaymentStrictReceive(
                    PathPaymentStrictReceiveResult::Success(success),
                ),
            ) => Some(calculate_exchange_rates(&success.offers, &success.last)),
            (
                OperationBody::PathPaymentStrictSend(_),
                OperationResultTr::PathPaymentStrictSend(PathPaymentStrictSendResult::Success(
                    success,
                )),
            ) => Some(calculate_exchange_rates(&success.offers, &success.last)),
            _ => None,
        }
    } else {
        None
    }
}

fn calculate_exchange_rates(
    offers: &VecM<ClaimAtom>,
    last: &SimplePaymentResult,
) -> Vec<ExchangeRate> {
    let mut rates = Vec::new();
    let mut usdc_amount = 0.0;

    for offer in offers.iter() {
        match offer {
            ClaimAtom::V0(atom) => {
                // Handle V0 atom
                process_claim_atom(
                    &mut rates,
                    &mut usdc_amount,
                    atom.asset_sold.clone(),
                    atom.amount_sold,
                    atom.asset_bought.clone(),
                    atom.amount_bought,
                );
            }
            ClaimAtom::OrderBook(atom) => {
                // Handle OrderBook atom
                process_claim_atom(
                    &mut rates,
                    &mut usdc_amount,
                    atom.asset_sold.clone(),
                    atom.amount_sold,
                    atom.asset_bought.clone(),
                    atom.amount_bought,
                );
            }
            ClaimAtom::LiquidityPool(atom) => {
                // Handle LiquidityPool atom
                process_claim_atom(
                    &mut rates,
                    &mut usdc_amount,
                    atom.asset_sold.clone(),
                    atom.amount_sold,
                    atom.asset_bought.clone(),
                    atom.amount_bought,
                );
            }
        }
    }

    // Handle the last payment
    process_last_payment(&mut rates, &mut usdc_amount, &last.asset, last.amount);

    rates
}

fn process_claim_atom(
    rates: &mut Vec<ExchangeRate>,
    usdc_amount: &mut f64,
    asset_sold: Asset,
    amount_sold: i64,
    asset_bought: Asset,
    amount_bought: i64,
) {
    if utils::asset_is_usdc(&asset_sold) {
        *usdc_amount = satoshi_amount(amount_sold);
        rates.push(ExchangeRate {
            asset: utils::format_asset(&asset_bought),
            usd_value: *usdc_amount / satoshi_amount(amount_bought),
        });
    } else if utils::asset_is_usdc(&asset_bought) {
        *usdc_amount = satoshi_amount(amount_bought);
        rates.push(ExchangeRate {
            asset: utils::format_asset(&asset_sold),
            usd_value: *usdc_amount / satoshi_amount(amount_sold),
        });
    }
}

fn process_last_payment(
    rates: &mut Vec<ExchangeRate>,
    usdc_amount: &mut f64,
    asset: &Asset,
    amount: i64,
) {
    if utils::asset_is_usdc(asset) || *usdc_amount > 0.0 {
        let usd_value = if utils::asset_is_usdc(asset) {
            amount as f64 / *usdc_amount
        } else {
            *usdc_amount / satoshi_amount(amount)
        };
        rates.push(ExchangeRate {
            asset: utils::format_asset(asset),
            usd_value,
        });
    }
}

fn satoshi_amount(amount: i64) -> f64 {
    let satoshis_in_whole_coin = 10_000_000.0;
    amount as f64 / satoshis_in_whole_coin
}
