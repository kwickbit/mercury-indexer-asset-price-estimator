pub(crate) mod exchange_rate;
pub(crate) mod models;
pub(crate) mod swap;

use zephyr_sdk::{DatabaseInteract, EnvClient};

use crate::db::swap::Swap;
use crate::db::models::{ExchangeRateMap, RatesDbRow, SwapDbRow};

pub fn save_swaps(client: &EnvClient, swaps: &[Swap]) {
    let timestamp = client.reader().ledger_timestamp();
    swaps
        .iter()
        .for_each(|swap| SwapDbRow::new(swap, timestamp).put(client));
}

pub fn save_rates(client: &EnvClient, rates: &ExchangeRateMap) -> bool {
    let should_save_rates = client.read::<RatesDbRow>().is_empty();

    if should_save_rates {
        rates.iter().for_each(|item| {
            RatesDbRow::from(item).put(client);
        });
    }

    should_save_rates
}
