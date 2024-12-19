use time::{format_description::well_known::Iso8601, PrimitiveDateTime};
use zephyr_sdk::prelude::TableQueryWrapper;

use super::RatesDbRow;

#[derive(Debug)]
pub(super) enum ExchangeRateError {
    DatabaseError,
    EmptyTransactions(String),
    EmptyRequest,
    InvalidDate,
    InvalidDateOrder,
    MissingIssuer(String),
    NotFound(String),
}

// This trait is implemented by types that contain an asset code and optional
// issuer that need normalization, particularly for the XLM/Native case
pub(super) trait NormalizeAssetIssuer {
    fn normalize_issuer(&self) -> Option<String>;
}

pub(super) fn normalize_issuer(asset_code: &str, asset_issuer: &Option<String>) -> Option<String> {
    if asset_code == "XLM" {
        Some("Native".to_string())
    } else {
        asset_issuer.clone()
    }
}

pub(super) fn parse_timestamp(date_str: &str) -> Result<i64, ExchangeRateError> {
    Ok(PrimitiveDateTime::parse(date_str, &Iso8601::DEFAULT)
        .map_err(|_| ExchangeRateError::InvalidDate)?
        .assume_utc()
        .unix_timestamp())
}

pub(super) fn query_db(
    mut query: TableQueryWrapper,
    timestamp: i64,
) -> Result<Vec<RatesDbRow>, ExchangeRateError> {
    let mut rows = query
        .column_lt("timestamp", timestamp)
        .read::<RatesDbRow>()
        .map_err(|_| ExchangeRateError::DatabaseError)?
        .into_iter()

        // Maybe it was some blunder during development, but some NaN exchange
        // rates crept into the DB. We filter them out.
        .filter(|row| row.rate.to_string() != "NaN")
        .collect::<Vec<RatesDbRow>>();

    rows.sort_by_key(|row| row.timestamp);
    Ok(rows)
}
