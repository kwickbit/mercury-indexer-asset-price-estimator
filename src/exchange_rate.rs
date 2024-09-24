use zephyr_sdk::EnvClient;

use crate::db::SwapDbRow;

pub fn calculate_exchange_rates(client: &EnvClient) -> String {
    let rows = client.read::<SwapDbRow>();

    let floatcoin_counts = rows
        .iter()
        .fold(std::collections::HashMap::new(), |mut counts, row| {
            *counts.entry(&row.floating).or_insert(0) += 1;
            counts
        });

    floatcoin_counts
        .iter()
        .map(|(coin, count)| format!("{} {}", count, coin))
        .collect::<Vec<_>>()
        .join(", ")
}
