use zephyr_sdk::{prelude::*, DatabaseDerive, EnvClient};

#[derive(Clone, DatabaseDerive)]
#[with_name("savepoint")]
pub struct Savepoint {
    pub savepoint: u64,
}
