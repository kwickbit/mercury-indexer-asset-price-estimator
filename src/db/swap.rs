use std::fmt::Display;

use zephyr_sdk::{prelude::*, soroban_sdk::xdr::OfferEntry, DatabaseDerive, EnvClient};

use crate::{
    config::CONVERSION_FACTOR,
    utils::{format_asset, is_stablecoin},
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
    pub numerator: i32,
    pub denom: i32,
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
    pub price_numerator: i32,
    pub price_denominator: i32,
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
        if is_stablecoin(&offer_entry.selling) {
            Swap {
                created_at: None,
                stablecoin: format_asset(&offer_entry.selling),
                stablecoin_amount: offer_entry.amount as f64,
                is_stablecoin_sale: true,
                floating_asset: format_asset(&offer_entry.buying),
                price_numerator: offer_entry.price.n,
                price_denominator: offer_entry.price.d,
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
                price_numerator: offer_entry.price.d,
                price_denominator: offer_entry.price.n,
            }
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
