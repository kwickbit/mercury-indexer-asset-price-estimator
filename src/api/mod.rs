pub(crate) mod extras;
pub(crate) mod rates_history;
pub(crate) mod shared;

use std::{cmp::Ordering::Equal, collections::HashMap};

use serde::{Deserialize, Serialize};
use time::format_description::well_known::Iso8601;
use zephyr_sdk::EnvClient;

use crate::{
    db::{exchange_rate::RatesDbRow, savepoint::Savepoint},
    utils::{is_certified_asset, parse_date},
};

use shared::query_db;

#[derive(Deserialize, Serialize)]
pub(crate) struct ExchangeRateRequest {
    asset_code: String,
    asset_issuer: Option<String>,
    date: Option<String>,
}

enum ExchangeRateError {
    DatabaseError,
    InvalidDate,
    NotFound(String),
}

struct ValidatedRequest {
    asset_code: String,
    asset_issuer: Option<String>,
    timestamp: Option<i64>,
}

#[no_mangle]
pub extern "C" fn get_exchange_rate() {
    let client = EnvClient::empty();
    let request = client.read_request_body::<ExchangeRateRequest>();

    let response = match handle_request(&client, &request) {
        Ok(data) => build_ok_response(data),
        Err(error) => build_error_response(error, &request),
    };

    client.conclude(&response);
}

fn handle_request(
    client: &EnvClient,
    request: &ExchangeRateRequest,
) -> Result<Vec<RatesDbRow>, ExchangeRateError> {
    let validated_request = validate_request(request)?;
    let db_results = query_database(client, &validated_request)?;
    process_results(db_results, &validated_request)
}

fn validate_request(request: &ExchangeRateRequest) -> Result<ValidatedRequest, ExchangeRateError> {
    let timestamp = match &request.date {
        Some(date_str) => Some(
            time::PrimitiveDateTime::parse(date_str, &Iso8601::DEFAULT)
                .map_err(|_| ExchangeRateError::InvalidDate)?
                .assume_utc()
                .unix_timestamp(),
        ),
        None => None,
    };

    // We don't allow non-native tokens named XLM.
    let asset_issuer = if request.asset_code == "XLM" {
        Some("Native".to_string())
    } else {
        request.asset_issuer.clone()
    };

    Ok(ValidatedRequest {
        asset_code: request.asset_code.clone(),
        asset_issuer,
        timestamp,
    })
}

fn query_database(
    client: &EnvClient,
    params: &ValidatedRequest,
) -> Result<Vec<RatesDbRow>, ExchangeRateError> {
    let mut query = client.read_filter();
    query.column_equal_to("floatcode", params.asset_code.clone());

    if let Some(issuer) = &params.asset_issuer {
        query.column_equal_to("fltissuer", issuer.clone());
    }

    let timestamp = match params.timestamp {
        Some(timestamp) => timestamp,
        None => {
            client
                .read::<Savepoint>()
                .first()
                .ok_or(ExchangeRateError::NotFound("timestamp".to_string()))?
                .savepoint as i64
        }
    };

    query_db(query, timestamp)
}

fn process_results(
    results: Vec<RatesDbRow>,
    request: &ValidatedRequest,
) -> Result<Vec<RatesDbRow>, ExchangeRateError> {
    // We keep only the most recent exchange rate for each issuer
    let processed_results = results.into_iter().fold(HashMap::new(), |mut acc, row| {
        if request.asset_issuer.as_ref() == Some(&row.fltissuer) || request.asset_issuer.is_none() {
            acc.entry(row.fltissuer.clone())
                .and_modify(|entry| *entry = row.clone())
                .or_insert(row);
        }
        acc
    });

    if processed_results.is_empty() {
        Err(ExchangeRateError::NotFound("exchange rate".to_string()))
    } else {
        Ok(processed_results.into_values().collect::<Vec<_>>())
    }
}

fn build_ok_response(
    rate_data: Vec<RatesDbRow>,
    request: &ExchangeRateRequest,
) -> serde_json::Value {
    let request_datetime = parse_date(
        &parse_timestamp(
            &request
                .date
                .clone()
                .unwrap_or("2024-12-18T16:01:00Z".to_string()),
        )
        .unwrap(),
    );

    serde_json::json!({
        "status": 200,
        "data": rate_data.into_iter().map(|row| {
            serde_json::json!({
                "asset_code": row.floatcode,
                "asset_issuer": row.fltissuer,
                "base_currency": "USD",
                "rate_date_time": row.timestamp_iso8601(),
                "rate_request_date_time": request_datetime,
                "exchange_rate": row.rate.to_string(),
                "soroswap_certified_asset": is_certified_asset(&row.floatcode, &row.fltissuer),
                "volume": row.volume.to_string(),
            })
        }).collect::<Vec<_>>(),
    })
}

fn build_error_response(error: ExchangeRateError) -> serde_json::Value {
    let (status, message) = match error {
        ExchangeRateError::InvalidDate => (
            400,
            // TODO: make the format a bit more flexible; at least allow for
            //       the use of a space between the day and hour
            "Invalid date format. Please use the format '2020-09-16T14:30:00'.",
        ),
        ExchangeRateError::NotFound(object) => (404, &*format!("No {object} found.")),
        ExchangeRateError::DatabaseError => (500, "An error occurred while querying the database."),
        _ => unreachable!(),
    };

    serde_json::json!({
        "status": status,
        "data": {
            "error": message,
        },
    })
}
