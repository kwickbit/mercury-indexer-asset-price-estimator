pub(crate) mod exchange_rate;
pub(crate) mod savepoint;
pub(crate) mod swap;

use zephyr_sdk::{DatabaseInteract, EnvClient};

use exchange_rate::RatesDbRow;
use savepoint::Savepoint;
use swap::{Swap, SwapDbRow};

use crate::config::RATE_UPDATE_INTERVAL;

pub(crate) fn save_swaps(client: &EnvClient, swaps: &[Swap]) {
    let timestamp = client.reader().ledger_timestamp();

    swaps
        .iter()
        .for_each(|swap| SwapDbRow::new(swap, timestamp).put(client));
}

pub(crate) fn save_rates(client: &EnvClient) {
    let savepoints = client.read::<Savepoint>();

    // We only create a savepoint if none is found, and update
    // otherwise. This means we should never have more than one.
    if savepoints.len() > 1 {
        client
            .log()
            .error("Error: more than one savepoint found.", None);
        return;
    }

    let current_timestamp = client.reader().ledger_timestamp();
    let latest_savepoint = savepoints
        .first()
        .map(|s| s.savepoint)
        .unwrap_or(current_timestamp - RATE_UPDATE_INTERVAL);

    // We check if it is time to save the exchange rates.
    let is_time_to_save_rates = if savepoints.is_empty() {
        // When we force-deploy there is no savepoint, so we create one.
        first_savepoint(client, current_timestamp);

        // In that case we always save the rates.
        true
    } else {
        // If there is a savepoint, we only save fresh rates if
        // the existing ones are stale.
        let are_rates_stale = current_timestamp - latest_savepoint > RATE_UPDATE_INTERVAL;

        if are_rates_stale {
            update_savepoint(client, current_timestamp, latest_savepoint);
        }

        are_rates_stale
    };

    if is_time_to_save_rates {
        // Whether the savepoint was defined or not, we calculate the rates
        // for the interval defined as RATE_UPDATE_INTERVAL.
        let rates = exchange_rate::calculate_exchange_rates(client, latest_savepoint);

        rates.iter().for_each(|item| {
            let mut row = RatesDbRow::from(item);
            row.timestamp = current_timestamp;
            row.put(client);
        });
    }
}

fn first_savepoint(client: &EnvClient, current_timestamp: u64) {
    let savepoint = Savepoint {
        savepoint: current_timestamp,
    };

    savepoint.put(client);
    client
        .log()
        .debug(&format!("First savepoint: {current_timestamp}"), None);
}

fn update_savepoint(client: &EnvClient, current_timestamp: u64, latest_savepoint: u64) {
    let savepoint = Savepoint {
        savepoint: current_timestamp,
    };

    if let Err(sdk_error) = client
        .update()
        .column_equal_to("savepoint", latest_savepoint)
        .execute(&savepoint)
    {
        client.log().debug(
            &format!(
                "Sequence {} failed to update savepoint: {sdk_error}",
                client.reader().ledger_sequence()
            ),
            None,
        )
    };
}
