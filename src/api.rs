use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use time::{format_description::well_known::Iso8601, OffsetDateTime};
use zephyr_sdk::EnvClient;

use crate::db::exchange_rate::RatesDbRow;

#[derive(Deserialize, Serialize)]
pub struct ExchangeRateRequest {
    asset: String,
    date: Option<String>,
}

enum ExchangeRateError {
    NotFound,
    InvalidDate,
}

#[no_mangle]
pub extern "C" fn get_exchange_rate() {
    let client = EnvClient::empty();
    let request = client.read_request_body::<ExchangeRateRequest>();

    // We find the latest exchange rate for the requested asset before the
    // requested date, or the latest exchange rate if no date is provided.
    let response = match find_asset_data(&client, &request) {
        Ok(data) => {
            let date_time = get_row_timestamp(&data);

            serde_json::json!({
                "asset": request.asset,
                "exchange_rate": data.rate.to_string(),
                "date_time": date_time,
            })
        }
        Err(ExchangeRateError::NotFound) => {
            let error = match request.date {
                Some(date) => format!("No exchange rate found before date {}.", date),
                None => "No exchange rate found.".to_string(),
            };

            serde_json::json!({
                "asset": request.asset,
                "error": error,
            })
        }
        Err(ExchangeRateError::InvalidDate) => {
            serde_json::json!({
                "error": "Invalid date format. Please use the format '2020-09-16T14:30:00'.",
            })
        }
    };

    client.conclude(&response);
}

fn get_row_timestamp(data: &RatesDbRow) -> String {
    OffsetDateTime::from_unix_timestamp(data.timestamp.unwrap() as i64)
        .unwrap()
        .format(&Iso8601::DEFAULT)
        .unwrap()
}

fn find_asset_data(
    client: &EnvClient,
    request: &ExchangeRateRequest,
) -> Result<RatesDbRow, ExchangeRateError> {
    let exchange_rates = client.read::<RatesDbRow>();

    match &request.date {
        Some(date_str) => {
            let maybe_date_time = time::PrimitiveDateTime::parse(date_str, &Iso8601::DEFAULT)
                .map(|dt| dt.assume_utc());

            match maybe_date_time {
                Ok(date_time) => exchange_rates
                    .into_iter()
                    .rev()
                    .filter(|row| row.floating == request.asset)
                    .find(|row| row.timestamp.unwrap() <= date_time.unix_timestamp() as u64)
                    .ok_or(ExchangeRateError::NotFound),
                Err(_) => Err(ExchangeRateError::InvalidDate),
            }
        }
        None => exchange_rates
            .into_iter()
            .rev()
            .find(|row| row.floating == request.asset)
            .ok_or(ExchangeRateError::NotFound),
    }
}

#[no_mangle]
pub extern "C" fn get_all_exchange_rates() {
    let client = EnvClient::empty();
    let exchange_rates = client.read::<RatesDbRow>();

    let response = serde_json::json!(exchange_rates.iter().fold(
        HashMap::new(),
        |mut acc: HashMap<String, HashMap<String, String>>, row| {
            acc.entry(row.floating.clone())
                .or_default()
                .insert(get_row_timestamp(row), row.rate.to_string());
            acc
        }
    ));

    client.conclude(&response);
}
