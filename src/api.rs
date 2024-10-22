use std::{cmp::Ordering::Equal, collections::HashMap};

use serde::{Deserialize, Serialize};
use time::format_description::well_known::Iso8601;
use zephyr_sdk::EnvClient;

use crate::{
    constants::soroswap_tokens::SOROSWAP_TOKENS,
    db::{exchange_rate::RatesDbRow, savepoint::Savepoint},
};

#[derive(Deserialize, Serialize)]
pub struct ExchangeRateRequest {
    asset_code: String,
    asset_issuer: Option<String>,
    date: Option<String>,
}

enum ExchangeRateError {
    DatabaseError,
    InvalidDate,
    NotFound,
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

    let response = match handle_request(&client, request) {
        Ok(data) => build_ok_response(data),
        Err(error) => build_error_response(error),
    };

    client.conclude(&response);
}

fn handle_request(
    client: &EnvClient,
    request: ExchangeRateRequest,
) -> Result<Vec<RatesDbRow>, ExchangeRateError> {
    let validated_request = validate_request(request)?;
    let db_results = query_database(client, &validated_request)?;
    process_results(db_results, &validated_request)
}

fn validate_request(request: ExchangeRateRequest) -> Result<ValidatedRequest, ExchangeRateError> {
    let timestamp = match request.date {
        Some(date_str) => Some(
            time::PrimitiveDateTime::parse(&date_str, &Iso8601::DEFAULT)
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
        request.asset_issuer
    };

    Ok(ValidatedRequest {
        asset_code: request.asset_code,
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
                .ok_or(ExchangeRateError::NotFound)?
                .savepoint as i64
        }
    };

    query
        .column_lt("timestamp", timestamp)
        .read::<RatesDbRow>()
        .map_err(|_| ExchangeRateError::DatabaseError)
}

fn process_results(
    results: Vec<RatesDbRow>,
    request: &ValidatedRequest,
) -> Result<Vec<RatesDbRow>, ExchangeRateError> {
    let processed_results = results.into_iter().fold(HashMap::new(), |mut acc, row| {
        if request.asset_issuer.as_ref() == Some(&row.fltissuer) || request.asset_issuer.is_none() {
            acc.entry(row.fltissuer.clone())
                .and_modify(|entry| *entry = row.clone())
                .or_insert(row);
        }
        acc
    });

    if processed_results.is_empty() {
        Err(ExchangeRateError::NotFound)
    } else {
        Ok(processed_results.into_values().collect::<Vec<_>>())
    }
}

fn build_ok_response(mut rate_data: Vec<RatesDbRow>) -> serde_json::Value {
    rate_data.sort_by(|a, b| b.volume.partial_cmp(&a.volume).unwrap_or(Equal));

    serde_json::json!({
        "status": 200,
        "data": rate_data.into_iter().map(|row| {
            serde_json::json!({
                "asset_code": row.floatcode,
                "asset_issuer": row.fltissuer,
                "base_currency": "USD",
                "date_time": row.timestamp_iso8601(),
                "exchange_rate": row.rate.to_string(),
                "soroswap_certified_asset": SOROSWAP_TOKENS.contains(&(&row.floatcode, &row.fltissuer)),
                "volume": row.volume.to_string(),
            })
        }).collect::<Vec<_>>(),
    })
}

fn build_error_response(error: ExchangeRateError) -> serde_json::Value {
    let (status, message) = match error {
        ExchangeRateError::InvalidDate => (
            400,
            "Invalid date format. Please use the format '2020-09-16T14:30:00'.",
        ),
        ExchangeRateError::NotFound => (404, "No exchange rate found."),
        ExchangeRateError::DatabaseError => (500, "An error occurred while querying the database."),
    };

    serde_json::json!({
        "status": status,
        "data": {
            "error": message,
        },
    })
}
