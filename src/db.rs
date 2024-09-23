use zephyr_sdk::{
    prelude::*,
    soroban_sdk::xdr::{ScString, ScVal},
    DatabaseDerive, EnvClient,
};

use crate::swap::Swap;

#[derive(DatabaseDerive, Clone)]
#[with_name("swaps")]
struct SwapDbRow {
    creation: ScVal,
    stable: ScVal,
    stableamt: ScVal,
    stbl_sold: ScVal,
    floating: ScVal,
    numerator: ScVal,
    denom: ScVal,
}

impl SwapDbRow {
    fn new(swap: &Swap, timestamp: u64) -> Self {
        Self {
            creation: ScVal::I64(timestamp.try_into().unwrap()),
            stable: ScVal::String(ScString(swap.stablecoin.clone().try_into().unwrap())),
            stableamt: ScVal::I64(swap.stablecoin_amount),
            stbl_sold: ScVal::Bool(swap.stablecoin_sold),
            floating: ScVal::String(ScString(
                swap.floating_asset.clone().try_into().unwrap(),
            )),
            numerator: ScVal::I32(swap.price_numerator),
            denom: ScVal::I32(swap.price_denominator),
        }
    }
}

pub fn do_db_stuff(client: EnvClient, swaps: Vec<Swap>) {
    let timestamp = client.reader().ledger_timestamp();
    swaps.iter().for_each(|swap| {
        SwapDbRow::new(swap, timestamp).put(&client);
    });
}
