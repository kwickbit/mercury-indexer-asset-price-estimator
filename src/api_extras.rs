use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use zephyr_sdk::EnvClient;

use crate::db::{exchange_rate::RatesDbRow, savepoint::Savepoint};

#[derive(Deserialize, Serialize)]
pub struct CatRequest {
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
pub extern "C" fn get_all_exchange_rates() {
    let client = EnvClient::empty();
    let exchange_rates = client.read::<RatesDbRow>();

    let response = exchange_rates
        .iter()
        .fold(
            HashMap::new(),
            |mut acc: HashMap<(&str, &str), Vec<RatesDbRow>>, row| {
                acc.entry((&row.floatcode, &row.fltissuer))
                    .or_default()
                    .push(row.clone());
                acc
            },
        )
        .into_iter()
        .map(|((asset_code, asset_issuer), rates)| {
            serde_json::json!({
                "asset_code": asset_code,
                "asset_issuer": asset_issuer,
                "rates": rates.into_iter().map(|row| {
                    serde_json::json!({
                        "date": row.timestamp_iso8601(),
                        "rate": row.rate.to_string()
                    })
                }).collect::<Vec<_>>()
            })
        })
        .collect::<Vec<_>>();

    // Create the final response
    client.conclude(response);
}
