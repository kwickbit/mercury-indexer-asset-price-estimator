use zephyr_sdk::EnvClient;

use crate::config::{FORCE_MILESTONE, MILESTONE_INTERVAL};
// use crate::swap::{format_swap, Swap};

pub fn log_milestone(client: &EnvClient, sequence: u32, results: usize) {
    let env_logger = client.log();
    let logger = move |args: &str| env_logger.debug(args, None);
    if sequence % MILESTONE_INTERVAL == 0 && FORCE_MILESTONE {
        logger(&format!("Sequence {sequence} with {results} transactions"));
    }
}

// pub fn log<'a>(client: &'a EnvClient, sequence: u32, swaps: &[Swap]) -> impl Fn(&str) + 'a {
//     let env_logger = client.log();
//     let logger = move |args: &str| env_logger.debug(args, None);

//     if sequence % MILESTONE_INTERVAL == 0 && (FORCE_MILESTONE || swaps.is_empty()) {
//         logger(&format!("Sequence {}", sequence));
//     }

//     if !swaps.is_empty() {
//         if PRINT_TRANSACTION_DETAILS {
//             swaps.iter().for_each(|swap| {
//                 logger(&format_swap(swap));
//             });
//         } else {
//             logger(&format!(
//                 "Sequence {sequence} had {} interesting transactions",
//                 swaps.len()
//             ));
//         }
//     }

//     logger
// }
