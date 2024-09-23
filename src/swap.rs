use std::fmt::Display;

use zephyr_sdk::soroban_sdk::xdr::OfferEntry;

use crate::{
    config::CONVERSION_FACTOR,
    utils::{format_asset, is_stablecoin},
};

#[derive(Debug, Clone)]
pub struct Swap {
    pub stablecoin: String,
    pub stablecoin_amount: f64,
    pub stablecoin_sold: bool,
    pub floating_asset: String,
    pub price_numerator: i32,
    pub price_denominator: i32,
}

impl Display for Swap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let direction = if self.stablecoin_sold {
            "(sell)"
        } else {
            "(buy)"
        };

        write!(
            f,
            "{direction} {} {} for {} at {}",
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
                stablecoin: format_asset(&offer_entry.selling),
                stablecoin_amount: offer_entry.amount as f64,
                stablecoin_sold: true,
                floating_asset: format_asset(&offer_entry.buying),
                price_numerator: offer_entry.price.n,
                price_denominator: offer_entry.price.d,
            }
        } else {
            let amount =
                offer_entry.amount as f64 * offer_entry.price.n as f64 / offer_entry.price.d as f64;
            Swap {
                stablecoin: format_asset(&offer_entry.buying),
                stablecoin_amount: amount,
                stablecoin_sold: false,
                floating_asset: format_asset(&offer_entry.selling),
                price_numerator: offer_entry.price.d,
                price_denominator: offer_entry.price.n,
            }
        }
    }
}
