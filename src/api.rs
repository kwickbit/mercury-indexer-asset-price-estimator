use serde::{Deserialize, Serialize};
use time::format_description::well_known::Iso8601;
use zephyr_sdk::{prelude::TableQueryWrapper, EnvClient};

use crate::db::{exchange_rate::RatesDbRow, savepoint::Savepoint};

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
        Ok(data) => build_ok_response(data, &request.asset),
        Err(ExchangeRateError::NotFound) => build_not_found_response(request),
        Err(ExchangeRateError::InvalidDate) => {
            serde_json::json!({
                "status": 400,
                "data": {
                    "error": "Invalid date format. Please use the format '2020-09-16T14:30:00'.",
                },
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
    let possible_request_date_time = request
        .date
        .as_ref()
        .map(|date_str| time::PrimitiveDateTime::parse(date_str, &Iso8601::DEFAULT));

    // There are three possibilities: the user has provided a valid date, an
    // invalid one or none at all.
    match possible_request_date_time {
        Some(Ok(request_date_time)) => {
            let timestamp = request_date_time.assume_utc().unix_timestamp();
            let filter = |query: &mut TableQueryWrapper| {
                query.column_lt("timestamp", timestamp);
            };
            find_rate(client, &request.asset, filter)
        }
        Some(Err(_)) => Err(ExchangeRateError::InvalidDate),
        None => find_latest_rate(client, &request.asset),
    }
}

fn find_latest_rate(client: &EnvClient, asset_code: &str) -> Result<RatesDbRow, ExchangeRateError> {
    let savepoints = client.read::<Savepoint>();

    let latest_savepoint = savepoints
        .first()
        .ok_or(ExchangeRateError::NotFound)?
        .savepoint;

    find_rate(client, asset_code, |query| {
        query.column_equal_to("timestamp", latest_savepoint);
    })
}

fn find_rate<F>(
    client: &EnvClient,
    asset_code: &str,
    filter: F,
) -> Result<RatesDbRow, ExchangeRateError>
where
    F: FnOnce(&mut TableQueryWrapper),
{
    let mut query = client.read_filter();
    filter(&mut query);
    let rates = query
        .column_equal_to("floating", asset_code.to_string())
        .read::<RatesDbRow>();

    match rates {
        Ok(rates) if !rates.is_empty() => Ok(rates.last().unwrap().clone()),
        _ => Err(ExchangeRateError::NotFound),
    }
}

fn build_ok_response(rate_data: RatesDbRow, asset: &str) -> serde_json::Value {
    let date_time = rate_data.timestamp_iso8601();

    serde_json::json!({
        "status": 200,
        "data": {
            "asset": asset,
            "base_currency": "USD",
            "date_time": date_time,
            "exchange_rate": rate_data.rate.to_string(),
        }
    })
}

fn build_not_found_response(request: ExchangeRateRequest) -> serde_json::Value {
    let error = match request.date {
        Some(date) => format!("No exchange rate found for date {}.", date),
        None => "No exchange rate found.".to_string(),
    };

    serde_json::json!({
        "status": 404,
        "data": {
            "asset": request.asset,
            "error": error,
        },
    })
}
