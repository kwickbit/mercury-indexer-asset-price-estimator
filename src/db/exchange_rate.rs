use std::collections::HashMap;

use zephyr_sdk::{prelude::*, DatabaseDerive, EnvClient};

use super::swap::SwapDbRow;
use crate::config::CONVERSION_FACTOR;

pub type UsdVolume = f64;
pub type WeightedSum = f64;
type ExchangeRate = f64;
pub type ExchangeRateMap = HashMap<String, (ExchangeRate, UsdVolume)>;

#[derive(Clone, DatabaseDerive)]
#[with_name("rates")]
pub struct RatesDbRow {
    pub timestamp: Option<u64>,
    pub floating: String,
    pub rate: f64,
    volume: f64,
}

impl From<(&String, &(f64, f64))> for RatesDbRow {
    fn from((floating, (rate, volume)): (&String, &(f64, f64))) -> Self {
        RatesDbRow {
            timestamp: None,
            floating: floating.clone(),
            rate: *rate,
            volume: *volume,
        }
    }
}

pub fn calculate_exchange_rates(client: &EnvClient, savepoint: u64) -> ExchangeRateMap {
    let rows: Vec<SwapDbRow> = client
        .read::<SwapDbRow>()
        .into_iter()
        .filter(|row| row.creation > savepoint)
        .collect();

    client.log().debug(
        &format!("Loaded {} swaps from the database", rows.len()),
        None,
    );

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
