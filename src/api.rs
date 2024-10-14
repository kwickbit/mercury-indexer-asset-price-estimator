use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use time::format_description::well_known::Iso8601;
use zephyr_sdk::{prelude::TableQueryWrapper, EnvClient};

use crate::db::{exchange_rate::RatesDbRow, savepoint::Savepoint};

#[derive(Deserialize, Serialize)]
pub struct ExchangeRateRequest {
    asset_code: String,
    asset_issuer: Option<String>,
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
        Ok(data) => build_ok_response(data),
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
// The user might provide an issuer as well; if they do, we look only
// for that issuer, otherwise we get all assets with that code.
fn find_exchange_rate(
    client: &EnvClient,
    request: &ExchangeRateRequest,
) -> Result<Vec<RatesDbRow>, ExchangeRateError> {
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

            find_rate(client, request, filter)
        }
        Some(Err(_)) => Err(ExchangeRateError::InvalidDate),
        None => find_latest_rate(client, request),
    }
}

fn find_latest_rate(
    client: &EnvClient,
    request: &ExchangeRateRequest,
) -> Result<Vec<RatesDbRow>, ExchangeRateError> {
    let savepoints = client.read::<Savepoint>();

    let latest_savepoint = savepoints
        .first()
        .ok_or(ExchangeRateError::NotFound)?
        .savepoint;

    find_rate(client, request, |query| {
        query.column_equal_to("timestamp", latest_savepoint);
    })
}

fn find_rate<F>(
    client: &EnvClient,
    request: &ExchangeRateRequest,
    filter: F,
) -> Result<Vec<RatesDbRow>, ExchangeRateError>
where
    F: FnOnce(&mut TableQueryWrapper),
{
    let mut query = client.read_filter();
    filter(&mut query);

    if let Some(issuer) = &request.asset_issuer {
        query.column_equal_to("fltissuer", issuer.to_string());
    }

    query
        .column_equal_to("floatcode", request.asset_code.to_string())
        .read::<RatesDbRow>()
        .map_err(|_| ExchangeRateError::NotFound)
}

fn build_ok_response(rate_data: Vec<RatesDbRow>) -> serde_json::Value {
    serde_json::json!({
        "status": 200,
        "data": rate_data.iter().map(|row| {
            HashMap::from([
                ("asset_code", row.floatcode.clone()),
                ("asset_issuer" , row.fltissuer.clone()),
                ("base_currency", "USD".into()),
                ("date_time", row.timestamp_iso8601()),
                ("exchange_rate", row.rate.to_string()),
            ])
        }).collect::<Vec<_>>(),
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
            "asset": request.asset_code,
            "error": error,
        },
    })
}
