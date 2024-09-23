use std::fmt::Display;

use zephyr_sdk::soroban_sdk::xdr::{OfferEntry, ScVal};

use crate::{
    config::CONVERSION_FACTOR,
    db::SwapDbRow,
    utils::{format_asset, is_stablecoin},
};

#[derive(Debug, Clone)]
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

impl TryFrom<SwapDbRow> for Swap {
    type Error = String;

    fn try_from(row: SwapDbRow) -> Result<Self, Self::Error> {
        let ScVal::I64(created_at) = row.creation else {
            return Err("Invalid created_at value".to_string());
        };

        let ScVal::String(ref stablecoin) = row.stable else {
            return Err("Invalid stablecoin value".to_string());
        };

        let stablecoin = stablecoin.to_string();

        let ScVal::I64(stablecoin_amount) = row.stableamt else {
            return Err("Invalid stablecoin_amount value".to_string());
        };

        let stablecoin_amount = stablecoin_amount as f64;

        let ScVal::Bool(stablecoin_sold) = row.stbl_sold else {
            return Err("Invalid stablecoin_sold value".to_string());
        };

        let ScVal::String(ref floating_asset) = row.floating else {
            return Err("Invalid floating_asset value".to_string());
        };

        let floating_asset = floating_asset.to_string();

        let ScVal::I32(price_numerator) = row.numerator else {
            return Err("Invalid price_numerator value".to_string());
        };

        let ScVal::I32(price_denominator) = row.denom else {
            return Err("Invalid price_denominator value".to_string());
        };

        Ok(Swap {
            created_at: Some(created_at as u64),
            stablecoin,
            stablecoin_amount,
            is_stablecoin_sale: stablecoin_sold,
            floating_asset,
            price_numerator,
            price_denominator,
        })
    }
}
