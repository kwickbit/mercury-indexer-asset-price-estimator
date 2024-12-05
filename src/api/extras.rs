use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use zephyr_sdk::EnvClient;

use crate::{
    config::CONVERSION_FACTOR,
    db::{exchange_rate::RatesDbRow, savepoint::Savepoint, swap::SwapDbRow},
    filter::Soroswap,
    utils::is_certified_asset,
};

#[derive(Deserialize, Serialize)]
pub(crate) struct CatRequest {
    text: String,
}

#[no_mangle]
pub extern "C" fn cat() {
    let client = EnvClient::empty();
    let request = client.read_request_body::<CatRequest>();
    client.log().debug(
        "=(^.^)= Called the =(^.^)= cat =(^.^)= function =(^.^)=",
        None,
    );
    client.conclude(&request.text);
}

#[no_mangle]
pub extern "C" fn savepoint() {
    let client = EnvClient::empty();
    let savepoint = client.read::<Savepoint>().first().unwrap().savepoint;
    client.conclude(serde_json::json!({ "savepoint": format!("{savepoint}") }));
}

#[no_mangle]
pub extern "C" fn get_all_currencies() {
    let client = EnvClient::empty();
    let exchange_rates = client.read::<RatesDbRow>();

    let mut currencies: Vec<String> = exchange_rates
        .iter()
        .map(|row| row.floatcode.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    currencies.sort();

    client.conclude(serde_json::json!(currencies));
}

#[no_mangle]
pub extern "C" fn get_soroswap_swaps() {
    let client = EnvClient::empty();
    let soroswap_swaps = client.read::<Soroswap>();

    let response = soroswap_swaps
        .iter()
        .map(|row| row.swap.clone())
        .collect::<Vec<_>>();

    client.conclude(serde_json::json!(response));
}

#[no_mangle]
pub extern "C" fn get_duplicate_swaps() {
    let client = EnvClient::empty();

    // Get the latest savepoint
    let savepoint = client.read::<Savepoint>().first().unwrap().savepoint;

    // Read all exchange rates at the latest savepoint
    let swaps = client
        .read_filter()
        .column_gt("creation", savepoint)
        .read::<SwapDbRow>()
        .unwrap();

    // Create a map to store swaps with the same characteristics
    use std::collections::HashMap;
    let mut swap_groups: HashMap<(String, String, String, String), Vec<u64>> = HashMap::new();

    // Group swaps by their characteristics and store creation times
    for rate in &swaps {
        let key = (
            rate.floatcode.clone(),
            rate.fltissuer.clone(),
            (rate.usdc_amnt as f64 / CONVERSION_FACTOR).to_string(),
            format!("{:.4}", rate.numerator as f64 / rate.denom as f64),
        );
        swap_groups.entry(key).or_default().push(rate.creation);
    }

    // Filter only duplicates (groups with more than 1 entry)
    let duplicates: Vec<_> = swap_groups
        .clone()
        .into_iter()
        .filter(|(_, creations)| creations.len() > 1)
        .map(|((asset_code, asset_issuer, volume, price), creations)| {
            serde_json::json!({
                "asset_code": asset_code,
                "asset_issuer": asset_issuer,
                "volume": volume,
                "price": price,
                "creation_times": creations
            })
        })
        .collect();

    client.conclude(serde_json::json!({
        "savepoint": savepoint,
        "duplicates": duplicates,
        "total_swaps": swaps.len(),
        "unique_swaps": swap_groups.len() - duplicates.len(),
    }));
}

#[no_mangle]
pub extern "C" fn get_all_exchange_rates() {
    let client = EnvClient::empty();
    let exchange_rates = client.read::<RatesDbRow>();

    let response = exchange_rates
        .iter()
        .fold(
            HashMap::new(),
            |mut acc: HashMap<(&str, &str), Vec<RatesDbRow>>, row| {
                // Look for the combo of asset code and issuer
                acc.entry((&row.floatcode, &row.fltissuer))
                    // If it's not there, create an empty Vec
                    .or_default()
                    // Append the new rate to the old ones (possibly empty)
                    .push(row.clone());
                acc
            },
        )
        .into_iter()
        .map(|((asset_code, asset_issuer), rates)| {
            serde_json::json!({
                "asset_code": asset_code,
                "asset_issuer": asset_issuer,
                "soroswap_certified_asset": is_certified_asset(asset_code, asset_issuer),
                "rates": rates.into_iter().map(|row| {
                    serde_json::json!({
                        "date": row.timestamp_iso8601(),
                        "rate": row.rate.to_string(),
                        "volume": row.volume.to_string(),
                    })
                }).collect::<Vec<_>>()
            })
        })
        .collect::<Vec<_>>();

    // Create the final response
    client.conclude(response);
}
