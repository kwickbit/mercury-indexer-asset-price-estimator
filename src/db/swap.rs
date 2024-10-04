use std::fmt::Display;

use zephyr_sdk::{
    prelude::*,
    soroban_sdk::xdr::{ClaimAtom, OfferEntry},
    DatabaseDerive, EnvClient,
};

use crate::{
    config::{CONVERSION_FACTOR, USDC},
    utils::{extract_claim_atom_data, format_asset},
};

#[derive(DatabaseDerive, Clone)]
#[with_name("swaps")]
pub struct SwapDbRow {
    pub creation: u64,
    pub stable: String,
    pub stableamt: i64,
    // This is a stand-in for a boolean: 1 means the swap was a
    // stablecoin sale, 0 means a purchase
    pub stbl_sold: i32,
    pub floating: String,
    pub numerator: i64,
    pub denom: i64,
}

impl SwapDbRow {
    pub fn new(swap: &Swap, timestamp: u64) -> Self {
        Self {
            creation: timestamp,
            stable: swap.stablecoin.clone(),
            stableamt: swap.stablecoin_amount as i64,
            stbl_sold: swap.is_stablecoin_sale as i32,
            floating: swap.floating_asset.clone(),
            numerator: swap.price_numerator,
            denom: swap.price_denominator,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Swap {
    pub created_at: Option<u64>,
    pub stablecoin: String,
    pub stablecoin_amount: f64,
    pub is_stablecoin_sale: bool,
    pub floating_asset: String,
    pub price_numerator: i64,
    pub price_denominator: i64,
}

impl Display for Swap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let direction = if self.is_stablecoin_sale {
            "(sell)"
        } else {
            "(buy)"
        };

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
            "{timestamp} {direction} {} {} for {} at {}",
            self.stablecoin_amount / CONVERSION_FACTOR,
            self.stablecoin,
            self.floating_asset,
            self.price_numerator as f64 / self.price_denominator as f64
        )
    }
}

impl From<OfferEntry> for Swap {
    fn from(offer_entry: OfferEntry) -> Self {
        if offer_entry.selling == USDC {
            Swap {
                created_at: None,
                stablecoin: format_asset(&offer_entry.selling),
                stablecoin_amount: offer_entry.amount as f64,
                is_stablecoin_sale: true,
                floating_asset: format_asset(&offer_entry.buying),
                price_numerator: offer_entry.price.n as i64,
                price_denominator: offer_entry.price.d as i64,
            }
        } else {
            let amount =
                offer_entry.amount as f64 * offer_entry.price.n as f64 / offer_entry.price.d as f64;
            Swap {
                created_at: None,
                stablecoin: format_asset(&offer_entry.buying),
                stablecoin_amount: amount,
                is_stablecoin_sale: false,
                floating_asset: format_asset(&offer_entry.selling),
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

        let stablecoin = "USDC".to_string();

        if *asset_sold == USDC {
            Ok(Swap {
                created_at: None,
                stablecoin,
                stablecoin_amount: amount_sold as f64,
                is_stablecoin_sale: true,
                floating_asset: format_asset(asset_bought),
                price_numerator: amount_bought,
                price_denominator: amount_sold,
            })
        } else if *asset_bought == USDC {
            Ok(Swap {
                created_at: None,
                stablecoin,
                stablecoin_amount: amount_bought as f64,
                is_stablecoin_sale: false,
                floating_asset: format_asset(asset_sold),
                price_numerator: amount_sold,
                price_denominator: amount_bought,
            })
        } else {
            Err("No USDC was swapped.".into())
        }
    }
}

impl From<&SwapDbRow> for Swap {
    fn from(row: &SwapDbRow) -> Self {
        Swap {
            created_at: Some(row.creation),
            stablecoin: row.stable.to_string(),
            stablecoin_amount: row.stableamt as f64,
            is_stablecoin_sale: row.stbl_sold == 1,
            floating_asset: row.floating.to_string(),
            price_numerator: row.numerator,
            price_denominator: row.denom,
        }
    }
}
