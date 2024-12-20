use zephyr_sdk::{prelude::*, DatabaseDerive, EnvClient};

#[derive(Clone, DatabaseDerive)]
#[with_name("savepoint")]
/**
 * On deployment, we save the current ledger timestamp. Then on every ledger
 * close, we check if enough time has passed since the last savepoint to save
 * the exchange rates. This window is defined in the config file.
 */
pub(crate) struct Savepoint {
    pub(crate) savepoint: u64,
}
