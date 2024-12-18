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

pub(super) fn query_db(
    mut query: TableQueryWrapper,
    timestamp: i64,
) -> Result<Vec<RatesDbRow>, ExchangeRateError> {
    let mut rows = query
        .column_lt("timestamp", timestamp)
        .read::<RatesDbRow>()
        .map_err(|_| ExchangeRateError::DatabaseError)?;

    rows.sort_by_key(|row| row.timestamp);
    Ok(rows)
}
