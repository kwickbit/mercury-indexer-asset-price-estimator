use std::collections::HashMap;

use zephyr_sdk::{prelude::*, DatabaseDerive, EnvClient};

use super::swap::Swap;

pub type UsdVolume = f64;
pub type WeightedSum = f64;
type ExchangeRate = f64;
pub type ExchangeRateMap = HashMap<String, (ExchangeRate, UsdVolume)>;

#[derive(DatabaseDerive, Clone)]
#[with_name("swaps")]
pub struct SwapDbRow {
    pub creation: u64,
    pub stable: String,
    pub stableamt: i64,
    // This is a stand-in for a boolean: 1 means the swap was a
    // stablecoin sale, 0 means a purchase
    pub stbl_sold: i8,
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
            stbl_sold: swap.is_stablecoin_sale as i8,
            floating: swap.floating_asset.clone(),
            numerator: swap.price_numerator,
            denom: swap.price_denominator,
        }
    }
}

#[derive(DatabaseDerive, Clone)]
#[with_name("rates")]
pub struct RatesDbRow {
    pub floating: String,
    pub rate: f64,
    volume: f64,
}

impl From<(&String, &(f64, f64))> for RatesDbRow {
    fn from((floating, (rate, volume)): (&String, &(f64, f64))) -> Self {
        RatesDbRow {
            floating: floating.clone(),
            rate: *rate,
            volume: *volume,
        }
    }
}