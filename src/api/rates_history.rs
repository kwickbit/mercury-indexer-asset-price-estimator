use serde::{Deserialize, Serialize};
use zephyr_sdk::EnvClient;

use crate::utils::parse_date;

use super::{
    shared::{normalize_issuer, parse_timestamp, query_db, ExchangeRateError, NormalizeAssetIssuer},
    RatesDbRow,
};

// Request structures
#[derive(Clone, Debug, Deserialize, Serialize)]
struct HistoryAsset {
    asset_code: String,
    asset_issuer: Option<String>,
}

impl HistoryAsset {
    fn validate(self) -> Result<ValidatedHistoryAsset, ExchangeRateError> {
        self.normalize_issuer()
            .ok_or(ExchangeRateError::MissingIssuer(self.asset_code.clone()))
            .map(|issuer| ValidatedHistoryAsset {
                asset_code: self.asset_code,
                asset_issuer: issuer,
            })
    }
}

impl NormalizeAssetIssuer for HistoryAsset {
    fn normalize_issuer(&self) -> Option<String> {
        normalize_issuer(&self.asset_code, &self.asset_issuer)
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct HistoryRequestTransactions {
    asset: HistoryAsset,
    transaction_dates: Vec<String>,
    unrealized_date: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct HistoryRequest {
    assets: Vec<HistoryRequestTransactions>,
}

// Intermediate structures
#[derive(Clone, Debug, Deserialize, Serialize)]
struct ValidatedHistoryAsset {
    asset_code: String,
    asset_issuer: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct HistoryTransactionsTimestamps {
    asset: ValidatedHistoryAsset,
    transaction_timestamps: Vec<i64>,
    unrealized_timestamp: i64,
}

#[derive(Debug, Deserialize, Serialize)]
struct ValidatedHistoryAssetsWithTimestamps {
    assets: Vec<HistoryTransactionsTimestamps>,
}

// Response structures
#[derive(Debug, Deserialize, Serialize)]
struct TransactionExchangeRate {
    transaction_date: String,
    exchange_rate_date: String,
    exchange_rate: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct SuccessfulAsset {
    asset: ValidatedHistoryAsset,
    transaction_rates: Vec<TransactionExchangeRate>,
    unrealized_rate: TransactionExchangeRate,
}

#[derive(Debug, Deserialize, Serialize)]
struct FailedAsset {
    asset: ValidatedHistoryAsset,
    error: String,
}

type AssetHistoryResult = Result<SuccessfulAsset, FailedAsset>;

#[derive(Debug, Deserialize, Serialize)]
struct ExchangeRateHistoryResponse {
    successful_assets: Vec<SuccessfulAsset>,
    failed_assets: Vec<FailedAsset>,
}

#[no_mangle]
pub extern "C" fn get_exchange_rate_history() {
    let client = EnvClient::empty();
    let request = client.read_request_body::<HistoryRequest>();

    let response = match handle_request(&request) {
        Ok(data) => build_ok_response(data),
        Err(error) => build_error_response(error),
    };

    client.conclude(&response);
}

fn handle_request(request: &HistoryRequest) -> Result<Vec<AssetHistoryResult>, ExchangeRateError> {
    validate_request(request)?
        .assets
        .iter()
        .map(process_asset)
        .collect()
}

fn validate_request(
    request: &HistoryRequest,
) -> Result<ValidatedHistoryAssetsWithTimestamps, ExchangeRateError> {
    if request.assets.is_empty() {
        return Err(ExchangeRateError::EmptyRequest);
    }

    let validated_assets = request
        .assets
        .iter()
        .map(validate_asset)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ValidatedHistoryAssetsWithTimestamps {
        assets: validated_assets,
    })
}

fn validate_asset(
    asset: &HistoryRequestTransactions,
) -> Result<HistoryTransactionsTimestamps, ExchangeRateError> {
    if asset.transaction_dates.is_empty() {
        return Err(ExchangeRateError::EmptyTransactions(
            asset.asset.asset_code.clone(),
        ));
    }

    let valid_identifier = asset.asset.clone().validate()?;
    let unrealized_timestamp = parse_timestamp(&asset.unrealized_date)?;

    let transaction_timestamps = asset
        .transaction_dates
        .iter()
        .map(|date| parse_timestamp(date))
        .collect::<Result<Vec<i64>, _>>()?;

    if unrealized_timestamp <= *transaction_timestamps.last().unwrap() {
        return Err(ExchangeRateError::InvalidDateOrder);
    }

    Ok(HistoryTransactionsTimestamps {
        asset: valid_identifier,
        transaction_timestamps,
        unrealized_timestamp,
    })
}

fn process_asset(
    asset: &HistoryTransactionsTimestamps,
) -> Result<AssetHistoryResult, ExchangeRateError> {
    let db_rows = query_database_for_one_asset(asset)?;

    if db_rows.is_empty() {
        return Err(ExchangeRateError::NotFound(format!(
            "No exchange rates found for {}:{}",
            asset.asset.asset_code, asset.asset.asset_issuer
        )));
    }

    let earliest_exchange_rate_timestamp = row_timestamp(db_rows.first().unwrap())?;
    let earliest_transaction_timestamp = *asset.transaction_timestamps.first().unwrap();

    if earliest_exchange_rate_timestamp > earliest_transaction_timestamp {
        return Ok(Err(FailedAsset {
            asset: asset.asset.clone(),
            error: "Found no exchange rates prior to the earliest transaction date".to_string(),
        }));
    }

    Ok(Ok(transaction_exchange_rates(asset, db_rows)?))
}

fn query_database_for_one_asset(
    asset: &HistoryTransactionsTimestamps,
) -> Result<Vec<RatesDbRow>, ExchangeRateError> {
    let client = EnvClient::empty();
    let mut query = client.read_filter();
    query.column_equal_to("floatcode", asset.asset.asset_code.clone());
    query.column_equal_to("fltissuer", asset.asset.asset_issuer.clone());

    query_db(query, asset.unrealized_timestamp)
}

fn row_timestamp(row: &RatesDbRow) -> Result<i64, ExchangeRateError> {
    row.timestamp
        .try_into()
        .map_err(|_| ExchangeRateError::DatabaseError)
}

fn transaction_exchange_rates(
    asset: &HistoryTransactionsTimestamps,
    db_rows: Vec<RatesDbRow>,
) -> Result<SuccessfulAsset, ExchangeRateError> {
    let unrealized_rate = TransactionExchangeRate {
        transaction_date: parse_date(&asset.unrealized_timestamp),
        exchange_rate_date: db_rows.last().unwrap().timestamp_iso8601(),
        exchange_rate: db_rows.last().unwrap().rate.to_string(),
    };

    let transaction_rates = build_transaction_rates(&asset.transaction_timestamps, db_rows);

    Ok(SuccessfulAsset {
        asset: asset.asset.clone(),
        transaction_rates,
        unrealized_rate,
    })
}

fn build_transaction_rates(
    transaction_timestamps: &[i64],
    db_rows: Vec<RatesDbRow>,
) -> Vec<TransactionExchangeRate> {
    // We have tested before; if the first row is higher than the first
    // transaction, that is an error. So we don't need to check it again.
    let mut possibly_too_late_row_index = 1;

    transaction_timestamps
        .iter()
        .fold(vec![], |mut acc, transaction_timestamp| {
            let is_within_bounds = possibly_too_late_row_index < db_rows.len();

            while is_within_bounds
                && row_timestamp(&db_rows[possibly_too_late_row_index]).unwrap()
                    < *transaction_timestamp
            {
                possibly_too_late_row_index += 1;
            }

            let last_row_before_transaction = &db_rows[possibly_too_late_row_index - 1];

            let exchange_rate = TransactionExchangeRate {
                transaction_date: parse_date(transaction_timestamp),
                exchange_rate_date: last_row_before_transaction.timestamp_iso8601(),
                exchange_rate: last_row_before_transaction.rate.to_string(),
            };

            acc.push(exchange_rate);
            acc
        })
}

fn build_ok_response(data: Vec<AssetHistoryResult>) -> serde_json::Value {
    let (assets, failed_assets): (Vec<_>, Vec<_>) =
        data.into_iter().partition(|asset| asset.is_ok());

    serde_json::json!({
        "status": 200,
        "data": {
            "assets": assets.iter().map(|asset| asset.as_ref().unwrap()).collect::<Vec<_>>(),
            "failed_assets": failed_assets.iter().map(|asset| asset.as_ref().unwrap()).collect::<Vec<_>>(),
        }
    })
}

fn build_error_response(error: ExchangeRateError) -> serde_json::Value {
    let (status, message) = match error {
        ExchangeRateError::EmptyRequest => (400, "Empty request. Must provide at least one asset."),
        ExchangeRateError::EmptyTransactions(asset) => (
            400,
            &*format!("Empty transaction dates. Must provide at least one transaction date for asset {asset}."),
        ),
        ExchangeRateError::InvalidDate => (
            400,
            // TODO: make the format a bit more flexible; at least allow for
            //       the use of a space between the day and hour
            "Invalid date format. Please use the format '2020-09-16T14:30:00'.",
        ),
        ExchangeRateError::InvalidDateOrder => (
            400,
            "Invalid date order. The last transaction date must be earlier than the unrealized gains date.",
        ),
        ExchangeRateError::MissingIssuer(asset) => (
            400,
            &*format!("Missing issuer. Must provide an issuer for the asset {asset}."),
        ),
        ExchangeRateError::NotFound(object) => (404, &*format!("No {object} found.")),
        ExchangeRateError::DatabaseError => (500, "An error occurred while querying the database."),
    };

    serde_json::json!({
        "status": status,
        "data": {
            "error": message,
        },
    })
}
