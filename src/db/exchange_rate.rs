use std::collections::HashMap;

use zephyr_sdk::{prelude::*, DatabaseDerive, EnvClient};

use super::swap::SwapDbRow;
use crate::{config::CONVERSION_FACTOR, utils::parse_date};

type UsdVolume = f64;
type WeightedSum = f64;
type ExchangeRate = f64;
pub(crate) type ExchangeRateMap = HashMap<String, (ExchangeRate, UsdVolume)>;

#[derive(Clone, DatabaseDerive)]
#[with_name("rates")]
pub(crate) struct RatesDbRow {
    pub(crate) timestamp: u64,
    pub(crate) floatcode: String,
    pub(crate) fltissuer: String,
    pub(crate) rate: f64,
    pub(crate) volume: f64,
}

impl RatesDbRow {
    pub(crate) fn timestamp_iso8601(&self) -> String {
        parse_date(&(self.timestamp as i64))
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

pub(crate) fn calculate_exchange_rates(client: &EnvClient, savepoint: u64) -> ExchangeRateMap {
    // We query the DB only for the swaps that happened after the savepoint
    let swaps = read_swaps(client, savepoint);

    client.log().debug(
        format!(
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
        // This could panic; it is crucial to us that it doesn't. One reason
        // it would is if there is too much data. To address that, we adjust
        // the interval between savepoints to something the DB can handle.
        .unwrap()
}

fn calculate_rates(swaps: Vec<SwapDbRow>) -> ExchangeRateMap {
    swaps
        .iter()
        .fold(HashMap::new(), extract_amounts)
        .into_iter()
        .filter_map(|(key, (weighted_sum, total_amount))| {
            let rate = weighted_sum / total_amount;
            if rate.is_nan() {
                EnvClient::empty().log().error(
                    format!(
                        "Found NaN exchange rate for asset {key}. Numerator is {weighted_sum} and denominator is {total_amount}."
                    ),
                    None,
                );
                None
            } else {
                Some((key, (rate, total_amount)))
            }
        })
        .collect::<ExchangeRateMap>()
}

// We calculate the total amounts of USDC and floatcoin swapped.
fn extract_amounts(
    mut counts: HashMap<String, (WeightedSum, UsdVolume)>,
    row: &SwapDbRow,
) -> HashMap<String, (WeightedSum, UsdVolume)> {
    let usdc_swapped: UsdVolume = row.usdc_amnt as f64 / CONVERSION_FACTOR;
    let swap_exchange_rate: f64 = row.numerator as f64 / row.denom as f64;

    // We group asset codes and issues into a single value for DB compatibility
    let floatcoin = format!("{}_{}", row.floatcode, row.fltissuer);

    // For XLM swaps, we sometimes get weird values, so we don't include them
    if swap_exchange_rate != 1e-7 {
        // Update the entry with a running sum of (weighted_sum, total_volume)
        counts
            .entry(floatcoin)
            .and_modify(|(floatcoin_total, total_volume)| {
                // The addition assignment += cannot overflow because the maximum
                // value of f64, 1.8e308, is ridiculously larger than the maximum
                // value of i64, 9.2e18.
                //
                // The trade-off, though, is that for values larger than 2^53 we
                // lose precision. This should not be a problem for calculating
                // exchange rates due to large volumes.
                *floatcoin_total += usdc_swapped * swap_exchange_rate;
                *total_volume += usdc_swapped;
            })
            .or_insert((usdc_swapped * swap_exchange_rate, usdc_swapped));
    }

    counts
}
