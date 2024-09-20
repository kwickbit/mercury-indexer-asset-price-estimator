use zephyr_sdk::EnvClient;

use crate::config::{FORCE_MILESTONE, MILESTONE_INTERVAL, PRINT_TRANSACTION_DETAILS};
use crate::swap::Swap;

pub fn log(client: &EnvClient, sequence: u32, swaps: &[Swap]) {
    let env_logger = client.log();
    let logger = move |args: &str| env_logger.debug(args, None);

    if sequence % MILESTONE_INTERVAL == 0 && (FORCE_MILESTONE || swaps.is_empty()) {
        logger(&format!("Sequence {}", sequence));
    }

    if !swaps.is_empty() {
        if PRINT_TRANSACTION_DETAILS {
            swaps.iter().for_each(|swap| {
                logger(&swap.to_string());
            });
        } else {
            logger(&format!(
                "Sequence {sequence} had {} interesting transactions",
                swaps.len()
            ));
        }
    }
}
