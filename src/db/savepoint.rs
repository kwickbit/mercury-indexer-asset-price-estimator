use zephyr_sdk::{prelude::*, DatabaseDerive, EnvClient};

#[derive(Clone, DatabaseDerive)]
#[with_name("savepoint")]
pub(crate) struct Savepoint {
    pub savepoint: u64,
}
