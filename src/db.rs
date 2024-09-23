use zephyr_sdk::{
    prelude::*,
    DatabaseDerive, EnvClient,
};

use crate::swap::Swap;

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
    fn new(swap: &Swap, timestamp: u64) -> Self {
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

pub fn save_swaps(client: &EnvClient, swaps: &[Swap]) {
    let timestamp = client.reader().ledger_timestamp();
    swaps
        .iter()
        .for_each(|swap| SwapDbRow::new(swap, timestamp).put(client));
}
