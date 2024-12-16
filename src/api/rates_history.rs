use serde::{Deserialize, Serialize};
use zephyr_sdk::EnvClient;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct AssetIdentifier {
    asset_code: String,
    asset_issuer: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct ExchangeRateHistoryAsset {
    asset: AssetIdentifier,
    transaction_dates: Vec<String>,
    unrealized_date: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct ExchangeRateHistoryRequest {
    assets: Vec<ExchangeRateHistoryAsset>,
}

#[no_mangle]
pub extern "C" fn get_exchange_rate_history() {
    let client = EnvClient::empty();
    let request = client.read_request_body::<ExchangeRateHistoryRequest>();
    client.conclude(&request);
}
