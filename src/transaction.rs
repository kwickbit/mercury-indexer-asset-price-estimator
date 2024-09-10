use zephyr_sdk::soroban_sdk::xdr::{
    Asset, ClaimAtom, Operation, OperationBody, OperationResult, OperationResultTr,
    PathPaymentStrictReceiveResult, PathPaymentStrictSendResult, SimplePaymentResult,
    TransactionEnvelope, TransactionResultMeta, TransactionResultResult, VecM,
};

use crate::utils;

pub struct InterestingTransaction {
    pub operations: Vec<Operation>,
    pub results: Vec<OperationResult>,
}

impl InterestingTransaction {
    pub fn new(envelope: &TransactionEnvelope, result_meta: &TransactionResultMeta) -> Self {
        let operations = utils::extract_transaction_operations(envelope);
        let results = match &result_meta.result.result.result {
            TransactionResultResult::TxSuccess(success) => success.to_vec(),
            _ => unreachable!(),
        };
        Self {
            operations: operations.to_vec(),
            results,
        }
    }

    pub fn exchange_rates(&self) -> Vec<ExchangeRate> {
        self.operations
            .iter()
            .zip(self.results.iter())
            .filter_map(|(op, result)| extract_exchange_rate(op, result))
            .flatten()
            .collect()
    }
}

pub struct ExchangeRate {
    pub asset: String,
    pub usd_value: f64,
}

fn extract_exchange_rate(
    operation: &Operation,
    result: &OperationResult,
) -> Option<Vec<ExchangeRate>> {
    if let OperationResult::OpInner(inner) = result {
        match (&operation.body, inner) {
            (OperationBody::PathPaymentStrictReceive(_), OperationResultTr::PathPaymentStrictReceive(PathPaymentStrictReceiveResult::Success(success))) => {
                Some(calculate_exchange_rates(&success.offers, &success.last))
            }
            (OperationBody::PathPaymentStrictSend(_), OperationResultTr::PathPaymentStrictSend(PathPaymentStrictSendResult::Success(success))) => {
                Some(calculate_exchange_rates(&success.offers, &success.last))
            }
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
    if utils::format_asset(&asset_sold) == "USDC" {
        *usdc_amount = amount_sold as f64 / 10_000_000.0;
        rates.push(ExchangeRate {
            asset: utils::format_asset(&asset_bought),
            usd_value: *usdc_amount / (amount_bought as f64 / 10_000_000.0),
        });
    } else if utils::format_asset(&asset_bought) == "USDC" {
        *usdc_amount = amount_bought as f64 / 10_000_000.0;
        rates.push(ExchangeRate {
            asset: utils::format_asset(&asset_sold),
            usd_value: *usdc_amount / (amount_sold as f64 / 10_000_000.0),
        });
    }
}

fn process_last_payment(
    rates: &mut Vec<ExchangeRate>,
    usdc_amount: &mut f64,
    asset: &Asset,
    amount: i64,
) {
    if utils::format_asset(asset) == "USDC" || *usdc_amount > 0.0 {
        let usd_value = if utils::format_asset(asset) == "USDC" {
            amount as f64 / *usdc_amount
        } else {
            *usdc_amount / (amount as f64 / 10_000_000.0)
        };
        rates.push(ExchangeRate {
            asset: utils::format_asset(asset),
            usd_value,
        });
    }
}
