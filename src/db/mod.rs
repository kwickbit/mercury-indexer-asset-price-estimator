pub(crate) mod exchange_rate;
pub(crate) mod savepoint;
pub(crate) mod swap;

use zephyr_sdk::{DatabaseInteract, EnvClient};

use exchange_rate::{ExchangeRateMap, RatesDbRow};
use swap::{Swap, SwapDbRow};

pub fn save_swaps(client: &EnvClient, swaps: &[Swap]) {
    let timestamp = client.reader().ledger_timestamp();
    swaps
        .iter()
        .for_each(|swap| SwapDbRow::new(swap, timestamp).put(client));
}

pub fn save_rates(client: &EnvClient, rates: &ExchangeRateMap) -> bool {
    let should_save_rates = client.read::<RatesDbRow>().is_empty();

    if should_save_rates {
        let ledger_timestamp = client.reader().ledger_timestamp();
        rates.iter().for_each(|item| {
            let mut row = RatesDbRow::from(item);
            row.timestamp = Some(ledger_timestamp);
            row.put(client);
        });
    }

    should_save_rates
}
