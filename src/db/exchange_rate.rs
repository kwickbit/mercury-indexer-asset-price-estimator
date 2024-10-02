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
    // We query the DB only for the swaps that happened after the savepoint
    let swaps = read_swaps(client, savepoint);

    client.log().debug(
        &format!(
            "Loaded {} swaps from the database ({} kb)",
            swaps.len(),
            (swaps.len() * std::mem::size_of::<SwapDbRow>()) / 1024
        ),
        None,
    );

    calculate_rates(swaps)
}

fn read_swaps(client: &EnvClient, savepoint: u64) -> Vec<SwapDbRow> {
    client
        .read_filter()
        .column_gt("creation", savepoint)
        .read::<SwapDbRow>()
        .unwrap()
}

fn calculate_rates(swaps: Vec<SwapDbRow>) -> ExchangeRateMap {
    swaps
        .iter()
        .fold(HashMap::new(), extract_amounts)
        .into_iter()
        .map(|(key, (weighted_sum, total_amount))| {
            (key.to_owned(), (weighted_sum / total_amount, total_amount))
        })
        .collect::<ExchangeRateMap>()
}

fn extract_amounts<'a>(
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
