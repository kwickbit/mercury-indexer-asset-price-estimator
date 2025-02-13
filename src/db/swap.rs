use std::fmt::Display;

use zephyr_sdk::{
    prelude::*,
    soroban_sdk::xdr::{AlphaNum12, AlphaNum4, Asset},
    DatabaseDerive, EnvClient,
};

use crate::{
    config::{CONVERSION_FACTOR, USDC, XLM_ADDRESS},
    utils::build_nonnative_swap_asset,
};

#[derive(Clone, DatabaseDerive)]
#[with_name("swaps")]
pub(crate) struct SwapDbRow {
    pub(crate) creation: u64,
    pub(crate) usdc_amnt: i64,
    pub(crate) floatcode: String,
    pub(crate) fltissuer: String,
    pub(crate) numerator: i64,
    pub(crate) denom: i64,
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SwapAsset {
    pub(crate) code: &'static str,
    pub(crate) issuer: &'static str,
    pub(crate) contract: &'static str,
}

impl TryFrom<&Asset> for SwapAsset {
    type Error = String;

    fn try_from(asset: &Asset) -> Result<Self, Self::Error> {
        match asset {
            Asset::Native => Ok(SwapAsset {
                code: "XLM",
                issuer: "Native",
                contract: XLM_ADDRESS,
            }),
            Asset::CreditAlphanum4(AlphaNum4 { asset_code, issuer }) => {
                build_nonnative_swap_asset(asset_code.as_slice(), issuer.to_string()).copied()
            }
            Asset::CreditAlphanum12(AlphaNum12 { asset_code, issuer }) => {
                build_nonnative_swap_asset(asset_code.as_slice(), issuer.to_string()).copied()
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct SwapData {
    pub(crate) amount_bought: i64,
    pub(crate) amount_sold: i64,
    pub(crate) asset_bought: Option<SwapAsset>,
    pub(crate) asset_sold: Option<SwapAsset>,
}

#[derive(Clone, Debug)]
pub(crate) struct Swap {
    pub(crate) created_at: Option<u64>,
    pub(crate) usdc_amount: f64,
    pub(crate) floating_asset_code: String,
    pub(crate) floating_asset_issuer: String,
    pub(crate) price_numerator: i64,
    pub(crate) price_denominator: i64,
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

impl TryFrom<&SwapData> for Swap {
    type Error = String;

    fn try_from(swap_data: &SwapData) -> Result<Self, Self::Error> {
        if swap_data.asset_sold.is_none() || swap_data.asset_bought.is_none() {
            return Err("Invalid asset in swap".to_string());
        }

        let asset_sold = swap_data.asset_sold.unwrap();
        let asset_bought = swap_data.asset_bought.unwrap();

        if asset_sold == USDC {
            Ok(Swap {
                created_at: None,
                usdc_amount: swap_data.amount_sold as f64,
                floating_asset_code: asset_bought.code.to_string(),
                floating_asset_issuer: asset_bought.issuer.to_string(),
                price_numerator: swap_data.amount_bought,
                price_denominator: swap_data.amount_sold,
            })
        } else if asset_bought == USDC {
            Ok(Swap {
                created_at: None,
                usdc_amount: swap_data.amount_bought as f64,
                floating_asset_code: asset_sold.code.to_string(),
                floating_asset_issuer: asset_sold.issuer.to_string(),
                price_numerator: swap_data.amount_sold,
                price_denominator: swap_data.amount_bought,
            })
        } else {
            Err("Swap does not involve USDC".to_string())
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

#[derive(Clone, DatabaseDerive)]
#[with_name("soroswaps")]
pub(crate) struct Soroswap {
    pub(crate) swap: String,
}

impl Soroswap {
    pub(crate) fn save(swap_data: &SwapData) {
        let asset_bought = swap_data.asset_bought.as_ref().unwrap();
        let asset_sold = swap_data.asset_sold.as_ref().unwrap();

        if *asset_bought == USDC || *asset_sold == USDC {
            let soroswap = Self {
                swap: format!(
                    "Swap data: {} {} for {} {}",
                    swap_data.amount_sold,
                    asset_sold.code,
                    swap_data.amount_bought,
                    asset_bought.code
                ),
            };

            soroswap.put(&EnvClient::new());
        }
    }
}
