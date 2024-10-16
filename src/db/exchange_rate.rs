use std::collections::HashMap;

use time::{format_description::well_known::Iso8601, OffsetDateTime};
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
    pub timestamp: u64,
    pub floatcode: String,
    pub fltissuer: String,
    pub rate: f64,
    pub volume: f64,
}

impl RatesDbRow {
    pub fn timestamp_iso8601(&self) -> String {
        OffsetDateTime::from_unix_timestamp(self.timestamp as i64)
            .unwrap()
            .format(&Iso8601::DEFAULT)
            .unwrap()
    }
}

impl From<(&String, &(f64, f64))> for RatesDbRow {
    fn from((floating_asset, (rate, volume)): (&String, &(f64, f64))) -> Self {
        let (floatcode, fltissuer) = floating_asset.split_once('_').unwrap();

        RatesDbRow {
            timestamp: 0,
            floatcode: floatcode.to_string(),
            fltissuer: fltissuer.to_string(),
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
    let the_cooler_client = EnvClient::empty();

    let metasyntactical_variable = client
        .read_filter()
        .column_gt("creation", savepoint)
        .read::<SwapDbRow>()
        .map_err(|error| {
            the_cooler_client.log().error(
                &format!("Error while reading swaps from the database: {error}"),
                None,
            );
            error
        })
        .unwrap();

    the_cooler_client.log().debug(
        &format!(
            "Read {} swaps from the database.",
            metasyntactical_variable.len()
        ),
        None,
    );
    metasyntactical_variable
}

fn calculate_rates(swaps: Vec<SwapDbRow>) -> ExchangeRateMap {
    swaps
        .iter()
        .fold(HashMap::new(), extract_amounts)
        .into_iter()
        .map(|(key, (weighted_sum, total_amount))| {
            (key, (weighted_sum / total_amount, total_amount))
        })
        .collect::<ExchangeRateMap>()
}

// Comment
fn extract_amounts(
    mut counts: HashMap<String, (WeightedSum, UsdVolume)>,
    row: &SwapDbRow,
) -> HashMap<String, (WeightedSum, UsdVolume)> {
    let amount: UsdVolume = row.usdc_amnt as f64 / CONVERSION_FACTOR;
    let rate: WeightedSum = row.numerator as f64 / row.denom as f64;
    let key = format!("{}_{}", row.floatcode, row.fltissuer);

    // For XLM swaps, we sometimes get weird values, so we don't include them
    if rate != 1e-7 {
        // Update the entry with a running sum of (weighted_sum, total_amount)
        counts
            .entry(key)
            .and_modify(|(weighted_sum, total_amount)| {
                *weighted_sum += amount * rate;
                *total_amount += amount;
            })
            .or_insert((amount * rate, amount));
    }

    counts
}
