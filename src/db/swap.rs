use std::fmt::Display;

use zephyr_sdk::{
    prelude::*,
    soroban_sdk::xdr::{ClaimAtom, OfferEntry},
    DatabaseDerive, EnvClient,
};

use crate::{
    config::{CONVERSION_FACTOR, USDC},
    utils::{
        extract_claim_atom_data, format_asset_code, format_asset_issuer, is_floating_asset_valid,
    },
};

#[derive(DatabaseDerive, Clone)]
#[with_name("swaps")]
pub(crate) struct SwapDbRow {
    pub creation: u64,
    pub usdc_amnt: i64,
    // This stands in for a bool: 1 means the swap was a USDC sale, 0 = purchase.
    pub usdc_sale: i32,
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
            usdc_sale: swap.is_usdc_sale as i32,
            floatcode: swap.floating_asset_code.clone(),
            fltissuer: swap.floating_asset_issuer.clone(),
            numerator: swap.price_numerator,
            denom: swap.price_denominator,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Swap {
    pub created_at: Option<u64>,
    pub usdc_amount: f64,
    pub is_usdc_sale: bool,
    pub floating_asset_code: String,
    pub floating_asset_issuer: String,
    pub price_numerator: i64,
    pub price_denominator: i64,
}

impl Display for Swap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let direction = if self.is_usdc_sale { "(sell)" } else { "(buy)" };

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
            "{timestamp} {direction} {} USDC for {} ({}) at {}",
            self.usdc_amount / CONVERSION_FACTOR,
            self.floating_asset_code,
            self.floating_asset_issuer,
            self.price_numerator as f64 / self.price_denominator as f64
        )
    }
}

impl From<OfferEntry> for Swap {
    fn from(offer_entry: OfferEntry) -> Self {
        if offer_entry.selling == USDC {
            let floating_asset = offer_entry.buying;
            Swap {
                created_at: None,
                usdc_amount: offer_entry.amount as f64,
                is_usdc_sale: true,
                floating_asset_code: format_asset_code(&floating_asset),
                floating_asset_issuer: format_asset_issuer(&floating_asset),
                price_numerator: offer_entry.price.n as i64,
                price_denominator: offer_entry.price.d as i64,
            }
        } else {
            let floating_asset = offer_entry.selling;
            let usdc_amount =
                offer_entry.amount as f64 * offer_entry.price.n as f64 / offer_entry.price.d as f64;

            Swap {
                created_at: None,
                usdc_amount,
                is_usdc_sale: false,
                floating_asset_code: format_asset_code(&floating_asset),
                floating_asset_issuer: format_asset_issuer(&floating_asset),
                price_numerator: offer_entry.price.d as i64,
                price_denominator: offer_entry.price.n as i64,
            }
        }
    }
}

impl TryFrom<&ClaimAtom> for Swap {
    type Error = String;
    fn try_from(claim_atom: &ClaimAtom) -> Result<Self, String> {
        let (asset_sold, amount_sold, asset_bought, amount_bought) =
            extract_claim_atom_data(claim_atom);

        if *asset_sold == USDC && is_floating_asset_valid(asset_bought) {
            Ok(Swap {
                created_at: None,
                usdc_amount: amount_sold as f64,
                is_usdc_sale: true,
                floating_asset_code: format_asset_code(asset_bought),
                floating_asset_issuer: format_asset_issuer(asset_bought),
                price_numerator: amount_bought,
                price_denominator: amount_sold,
            })
        } else if *asset_bought == USDC && is_floating_asset_valid(asset_sold) {
            Ok(Swap {
                created_at: None,
                usdc_amount: amount_bought as f64,
                is_usdc_sale: false,
                floating_asset_code: format_asset_code(asset_sold),
                floating_asset_issuer: format_asset_issuer(asset_sold),
                price_numerator: amount_sold,
                price_denominator: amount_bought,
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
            is_usdc_sale: row.usdc_sale == 1,
            floating_asset_code: row.floatcode.to_string(),
            floating_asset_issuer: row.fltissuer.to_string(),
            price_numerator: row.numerator,
            price_denominator: row.denom,
        }
    }
}
