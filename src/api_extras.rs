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
pub extern "C" fn get_all_exchange_rates() {
    let client = EnvClient::empty();
    let exchange_rates = client.read::<RatesDbRow>();

    let response = serde_json::json!(exchange_rates.iter().fold(
        HashMap::new(),
        |mut acc: HashMap<String, HashMap<String, String>>, row| {
            acc.entry(row.floatcode.clone())
                .or_default()
                .insert(row.timestamp_iso8601(), row.rate.to_string());
            acc
        }
    ));

    client.conclude(&response);
}
