use std::collections::HashMap;

use zephyr_sdk::EnvClient;

use crate::config::CONVERSION_FACTOR;
use super::models::{ExchangeRateMap, SwapDbRow, UsdVolume, WeightedSum};

pub fn calculate_exchange_rates(client: &EnvClient) -> ExchangeRateMap {
    let rows = client.read::<SwapDbRow>();
    client.log().debug(&format!("Loaded {} swaps from the database", rows.len()), None);

    let exchange_rates = rows
        .iter()
        .fold(HashMap::new(), extract_rates)
        // Now that we have the total swapped amounts, we calculate the exchange rate
        .into_iter()
        .map(|(key, (weighted_sum, total_amount))| {
            (key.to_owned(), (weighted_sum / total_amount, total_amount))
        })
        .collect::<ExchangeRateMap>();

    exchange_rates
}

fn extract_rates<'a>(
    mut counts: HashMap<&'a String, (WeightedSum, UsdVolume)>,
    row: &'a SwapDbRow,
) -> HashMap<&'a String, (WeightedSum, UsdVolume)> {
    let amount: UsdVolume = row.stableamt as f64 / CONVERSION_FACTOR;
    let rate: WeightedSum = row.numerator as f64 / row.denom as f64;

    // For XLM swaps, we sometimes get weird values, so we don't include them
    if rate != 1e-7 {
        // Update the entry with a running sum of (weighted_sum, total_amount)
        counts
            .entry(&row.floating)
            .and_modify(|(weighted_sum, total_amount)| {
                *weighted_sum += amount * rate;
                *total_amount += amount;
            })
            .or_insert((amount * rate, amount));
    }

    counts
}
