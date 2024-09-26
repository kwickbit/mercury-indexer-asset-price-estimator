pub(crate) mod exchange_rate;
pub(crate) mod savepoint;
pub(crate) mod swap;

use zephyr_sdk::{DatabaseInteract, EnvClient};

use exchange_rate::{ExchangeRateMap, RatesDbRow};
use savepoint::Savepoint;
use swap::{Swap, SwapDbRow};

use crate::config::RATE_UPDATE_INTERVAL;

pub fn save_swaps(client: &EnvClient, swaps: &[Swap]) {
    let timestamp = client.reader().ledger_timestamp();
    swaps
        .iter()
        .for_each(|swap| SwapDbRow::new(swap, timestamp).put(client));
}

pub fn save_rates(client: &EnvClient, rates: &ExchangeRateMap) {
    // This panics right when we first deploy and there is no savepoint.
    // That is acceptable.
    let latest_savepoint = client.read::<Savepoint>()[0].savepoint;
    let current_timestamp = client.reader().ledger_timestamp();
    let should_save_rates = current_timestamp - latest_savepoint > RATE_UPDATE_INTERVAL;

    if should_save_rates {
        rates.iter().for_each(|item| {
            let mut row = RatesDbRow::from(item);
            row.timestamp = Some(current_timestamp);
            row.put(client);
        });
    }
}
