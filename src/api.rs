use serde::{Deserialize, Serialize};
use time::{error::Parse, format_description::well_known::Iso8601, OffsetDateTime};
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

    let response = match find_exchange_rate(&client, &request) {
        Ok(data) => build_ok_response(data, &request),
        Err(ExchangeRateError::NotFound) => build_not_found_response(request),
        Err(ExchangeRateError::InvalidDate) => {
            serde_json::json!({
                "error": "Invalid date format. Please use the format '2020-09-16T14:30:00'.",
            })
        }
    };

    client.conclude(&response);
}

// We find the latest exchange rate for the requested asset before the
// requested date, or the latest exchange rate if no date is provided.
fn find_exchange_rate(
    client: &EnvClient,
    request: &ExchangeRateRequest,
) -> Result<RatesDbRow, ExchangeRateError> {
    let exchange_rates = client.read::<RatesDbRow>();

    match &request.date {
        Some(date_str) => {
            let maybe_date_time = time::PrimitiveDateTime::parse(date_str, &Iso8601::DEFAULT)
                .map(|dt| dt.assume_utc());

            find_rate_with_date(maybe_date_time, &exchange_rates, request)
        }
        None => exchange_rates
            .into_iter()
            .rev()
            .find(|row| row.floating == request.asset)
            .ok_or(ExchangeRateError::NotFound),
    }
}

fn build_ok_response(data: RatesDbRow, request: &ExchangeRateRequest) -> serde_json::Value {
    let date_time = data.timestamp_iso8601();

    serde_json::json!({
        "asset": request.asset,
        "exchange_rate": data.rate.to_string(),
        "date_time": date_time,
    })
}

fn build_not_found_response(request: ExchangeRateRequest) -> serde_json::Value {
    let error = match request.date {
        Some(date) => format!("No exchange rate found before date {}.", date),
        None => "No exchange rate found.".to_string(),
    };

    serde_json::json!({
        "asset": request.asset,
        "error": error,
    })
}

fn find_rate_with_date(
    maybe_date_time: Result<OffsetDateTime, Parse>,
    exchange_rates: &[RatesDbRow],
    request: &ExchangeRateRequest,
) -> Result<RatesDbRow, ExchangeRateError> {
    match maybe_date_time {
        Ok(date_time) => exchange_rates
            .iter()
            .rev()
            .filter(|row| row.floating == request.asset)
            .find(|row| row.timestamp.unwrap() <= date_time.unix_timestamp() as u64)
            .cloned()
            .ok_or(ExchangeRateError::NotFound),
        Err(_) => Err(ExchangeRateError::InvalidDate),
    }
}
