#![warn(missing_docs)]

pub(crate) mod extras;
pub(crate) mod rates_history;
pub(crate) mod shared;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use zephyr_sdk::EnvClient;

use crate::{
    db::{exchange_rate::RatesDbRow, savepoint::Savepoint},
    utils::is_certified_asset,
};
use shared::{
    normalize_issuer, parse_timestamp, query_db, ExchangeRateError, NormalizeAssetIssuer,
};

#[derive(Deserialize, Serialize)]
pub(crate) struct ExchangeRateRequest {
    asset_code: String,
    asset_issuer: Option<String>,
    date: Option<String>,
}

impl NormalizeAssetIssuer for ExchangeRateRequest {
    fn normalize_issuer(&self) -> Option<String> {
        normalize_issuer(&self.asset_code, &self.asset_issuer)
    }
}

struct ValidatedRequest {
    asset_code: String,
    asset_issuer: Option<String>, // None means the asset is native (XLM)
    timestamp: Option<i64>,       // Seconds since the Unix epoch; None means that the most recent
                                  // exchange rates will be retrieved
}

/// Retrieves the USD exchange rate for a given asset.
///
/// Returns the latest exchange rate for the specified asset, up to an optional
/// given time. For non-native assets (not XLM), an issuer may be specified;
/// otherwise, all assets with the same code are retrieved.
///
/// # Request Format
/// ```json
/// {
///     "asset_code": "XLM",
///     "asset_issuer": "optional_issuer",
///     "date": "optional_ISO8601_timestamp"  // e.g. "2024-01-01T00:00:00"
/// }
/// ```
///
/// # Response Format
/// On success (status 200):
/// ```json
/// {
///     "status": 200,
///     "data": [{
///         "asset_code": "XLM",
///         "asset_issuer": "Native",
///         "base_currency": "USD",
///         "rate_date_time": "2023-12-31T23:58:30",
///         "exchange_rate": "1.2345",
///         "soroswap_certified_asset": true,
///         "volume": "10000.0"
///     }]
/// }
/// ```
///
/// # Errors
/// - 400: Invalid date format
/// - 404: No exchange rate found
/// - 500: Database error
#[no_mangle]
pub extern "C" fn get_exchange_rate() {
    let client = EnvClient::empty();
    let request = client.read_request_body::<ExchangeRateRequest>();

    let response = match handle_request(&request) {
        Ok(data) => build_ok_response(data),
        Err(error) => build_error_response(error),
    };

    client.conclude(&response);
}

fn handle_request(request: &ExchangeRateRequest) -> Result<Vec<RatesDbRow>, ExchangeRateError> {
    let validated_request = validate_request(request)?;
    let db_results = query_database(&validated_request)?;
    process_results(db_results, &validated_request)
}

fn validate_request(request: &ExchangeRateRequest) -> Result<ValidatedRequest, ExchangeRateError> {
    let timestamp = match &request.date {
        Some(date_str) => Some(parse_timestamp(date_str)?),
        None => None,
    };

    // We don't allow non-native tokens named XLM.
    let asset_issuer = request.normalize_issuer();

    Ok(ValidatedRequest {
        asset_code: request.asset_code.clone(),
        asset_issuer,
        timestamp,
    })
}

fn query_database(params: &ValidatedRequest) -> Result<Vec<RatesDbRow>, ExchangeRateError> {
    let client = EnvClient::empty();
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

fn build_ok_response(rate_data: Vec<RatesDbRow>) -> serde_json::Value {
    serde_json::json!({
        "status": 200,
        "data": rate_data.into_iter().map(|row| {
            serde_json::json!({
                "asset_code": row.floatcode,
                "asset_issuer": row.fltissuer,
                "base_currency": "USD",
                "rate_date_time": row.timestamp_iso8601(),
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
        // Other error types can only happen in the batch exchange rate endpoint.
        _ => unreachable!(),
    };

    serde_json::json!({
        "status": status,
        "data": {
            "error": message,
        },
    })
}
