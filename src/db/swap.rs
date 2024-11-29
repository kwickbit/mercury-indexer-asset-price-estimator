use std::fmt::Display;

use zephyr_sdk::{
    prelude::*,
    soroban_sdk::xdr::Asset,
    DatabaseDerive, EnvClient,
};

use crate::{
    config::{CONVERSION_FACTOR, USDC},
    utils::{
        format_asset_code, format_asset_issuer, is_floating_asset_valid,
    },
};

#[derive(Clone, DatabaseDerive)]
#[with_name("swaps")]
pub(crate) struct SwapDbRow {
    pub creation: u64,
    pub usdc_amnt: i64,
    pub floatcode: String,
    pub fltissuer: String,
    pub numerator: i64,
    pub denom: i64,
}

impl SwapDbRow {
    pub(crate) fn new(swap: &Swap, timestamp: u64) -> Self {
        Self {
            creation: timestamp,
            usdc_amnt: swap.usdc_amount as i64,
            floatcode: swap.floating_asset_code.clone(),
            fltissuer: swap.floating_asset_issuer.clone(),
            numerator: swap.price_numerator,
            denom: swap.price_denominator,
        }
    }
}

pub(crate) struct SwapData<'a> {
    pub asset_sold: &'a Asset,
    pub amount_sold: i64,
    pub asset_bought: &'a Asset,
    pub amount_bought: i64,
}

#[derive(Clone, Debug)]
pub(crate) struct Swap {
    pub created_at: Option<u64>,
    pub usdc_amount: f64,
    pub floating_asset_code: String,
    pub floating_asset_issuer: String,
    pub price_numerator: i64,
    pub price_denominator: i64,
}

impl Display for Swap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let timestamp = self
            .created_at
            .map(|ts| {
                let datetime = time::OffsetDateTime::from_unix_timestamp(ts as i64).unwrap();
                let format = time::format_description::parse(
                    "[year]-[month]-[day] [hour]:[minute]:[second]",
                )
                .unwrap();

                format!("[{}]", datetime.format(&format).unwrap())
            })
            .unwrap_or("(date error)".to_string());

        write!(
            f,
            "{timestamp}: {} USDC for {} ({}) at {}",
            self.usdc_amount / CONVERSION_FACTOR,
            self.floating_asset_code,
            self.floating_asset_issuer,
            self.price_numerator as f64 / self.price_denominator as f64
        )
    }
}

impl TryFrom<&SwapData<'_>> for Swap {
    type Error = String;
    fn try_from(swap_data: &SwapData) -> Result<Self, String> {
        if *swap_data.asset_sold == USDC && is_floating_asset_valid(swap_data.asset_bought) {
            Ok(Swap {
                created_at: None,
                usdc_amount: swap_data.amount_sold as f64,
                floating_asset_code: format_asset_code(swap_data.asset_bought),
                floating_asset_issuer: format_asset_issuer(swap_data.asset_bought),
                price_numerator: swap_data.amount_bought,
                price_denominator: swap_data.amount_sold,
            })
        } else if *swap_data.asset_bought == USDC && is_floating_asset_valid(swap_data.asset_sold) {
            Ok(Swap {
                created_at: None,
                usdc_amount: swap_data.amount_bought as f64,
                floating_asset_code: format_asset_code(swap_data.asset_sold),
                floating_asset_issuer: format_asset_issuer(swap_data.asset_sold),
                price_numerator: swap_data.amount_sold,
                price_denominator: swap_data.amount_bought,
            })
        } else {
            Err("Cannot create swap: no USDC involved, or invalid counterasset.".into())
        }
    }
}

impl From<&SwapDbRow> for Swap {
    fn from(row: &SwapDbRow) -> Self {
        Swap {
            created_at: Some(row.creation),
            usdc_amount: row.usdc_amnt as f64,
            floating_asset_code: row.floatcode.to_string(),
            floating_asset_issuer: row.fltissuer.to_string(),
            price_numerator: row.numerator,
            price_denominator: row.denom,
        }
    }
}
