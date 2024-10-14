use std::collections::HashMap;

use itertools::Itertools;
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
pub extern "C" fn get_all_exchange_rates() {
    let client = EnvClient::empty();
    let exchange_rates = client.read::<RatesDbRow>();

    let response = exchange_rates
        .into_iter()
        .chunk_by(|row| (row.floatcode.clone(), row.fltissuer.clone()))
        .into_iter()
        .map(|((asset_code, asset_issuer), group)| {
            serde_json::json!({
                "asset_code": asset_code,
                "asset_issuer": asset_issuer,
                "rates": group.map(rates_from_row).collect::<Vec<_>>()
            })
        })
        .collect::<Vec<_>>();

    // Create the final response
    client.conclude(response);
}

fn rates_from_row(row: RatesDbRow) -> HashMap<String, String> {
    HashMap::from([
        ("date".to_string(), row.timestamp_iso8601()),
        ("rate".to_string(), row.rate.to_string()),
    ])
}
