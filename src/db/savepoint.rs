use zephyr_sdk::{prelude::*, DatabaseDerive, EnvClient};

#[derive(DatabaseDerive, Clone)]
#[with_name("savepoint")]
pub struct Savepoint {
    pub savepoint: u64,
}