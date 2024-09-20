use std::fmt::Display;

use crate::config::CONVERSION_FACTOR;

#[derive(Debug, Clone)]
pub struct Swap {
    pub stablecoin: String,
    pub stablecoin_amount: i64,
    pub floating_asset: String,
    pub price_numerator: i32,
    pub price_denominator: i32,
}

impl Display for Swap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} for {} at {}",
            self.stablecoin_amount as f64 / CONVERSION_FACTOR,
            self.stablecoin,
            self.floating_asset,
            self.price_numerator as f64 / self.price_denominator as f64
        )
    }
}
