use zephyr_sdk::{
    prelude::*,
    soroban_sdk::xdr::{ScString, ScVal},
    DatabaseDerive, EnvClient,
};

use crate::swap::Swap;

#[derive(DatabaseDerive, Clone)]
#[with_name("swaps")]
pub struct SwapDbRow {
    pub creation: ScVal,
    pub stable: ScVal,
    pub stableamt: ScVal,
    pub stbl_sold: ScVal,
    pub floating: ScVal,
    pub numerator: ScVal,
    pub denom: ScVal,
}

impl SwapDbRow {
    fn new(swap: &Swap, timestamp: u64) -> Self {
        Self {
            creation: ScVal::I64(timestamp.try_into().unwrap()),
            stable: ScVal::String(ScString(swap.stablecoin.clone().try_into().unwrap())),
            stableamt: ScVal::I64(swap.stablecoin_amount as i64),
            stbl_sold: ScVal::Bool(swap.is_stablecoin_sale),
            floating: ScVal::String(ScString(swap.floating_asset.clone().try_into().unwrap())),
            numerator: ScVal::I32(swap.price_numerator),
            denom: ScVal::I32(swap.price_denominator),
        }
    }
}

pub fn save_swaps(client: &EnvClient, swaps: &[Swap]) {
    let timestamp = client.reader().ledger_timestamp();
    swaps
        .iter()
        .for_each(|swap| SwapDbRow::new(swap, timestamp).put(client));
}
