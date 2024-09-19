use zephyr_sdk::{
    prelude::*,
    soroban_sdk::xdr::{ScString, ScVal},
    DatabaseDerive, EnvClient,
};

use crate::swap::Swap;

#[derive(DatabaseDerive, Clone)]
#[with_name("storage")]
struct StorageTable {
    is_stored: ScVal,
    envelope: ScVal,
    res_meta: ScVal,
}

pub fn do_old_db_stuff(client: EnvClient, sequence: u32, results: usize) {
    let table = StorageTable {
        is_stored: ScVal::Bool(true),
        envelope: ScVal::U32(sequence),
        res_meta: ScVal::U32(results.try_into().unwrap()),
    };
    table.put(&client);
}

#[derive(DatabaseDerive, Clone)]
#[with_name("swaps")]
struct SwapDbRow {
    created_at: ScVal,
    stablecoin: ScVal,
    stablecoin_amount: ScVal,
    floating_asset: ScVal,
    price_numerator: ScVal,
    price_denominator: ScVal,
}

impl SwapDbRow {
    fn new(swap: &Swap, timestamp: i64) -> Self {
        Self {
            created_at: ScVal::I64(timestamp),
            stablecoin: ScVal::String(ScString(swap.stablecoin.clone().try_into().unwrap())),
            stablecoin_amount: ScVal::I64(swap.stablecoin_amount),
            floating_asset: ScVal::String(ScString(
                swap.floating_asset.clone().try_into().unwrap(),
            )),
            price_numerator: ScVal::I32(swap.price_numerator),
            price_denominator: ScVal::I32(swap.price_denominator),
        }
    }
}

pub fn do_db_stuff(client: EnvClient, swaps: Vec<Swap>) {
    let timestamp = chrono::Utc::now().timestamp();
    swaps.iter().for_each(|swap| {
        SwapDbRow::new(swap, timestamp).put(&client);
    });
}
