// The stablecoins we focus on
pub const STABLECOINS: [&str; 3] = ["USDC", "USDT", "DAI"];

// Amounts are represented with this factor
pub const CONVERSION_FACTOR: f64 = 10_000_000.0;

/*
The following constants are mostly for convenience.
These things were changing very quickly during early development,
so it made sense to group them here.
*/

// Do I want milestones to be printed even when there are transactions?
pub const FORCE_MILESTONE: bool = true;

// How often should milestones be printed?
// There are 11-12 sequences every minute.
pub const MILESTONE_INTERVAL: u32 = 2;

// Whether to print details of interesting transactions
// We always print at least the number of interesting transactions.
pub const PRINT_TRANSACTION_DETAILS: bool = true;

// Whether to interact with the DB at all
pub const DO_DB_STUFF: bool = false;

// When filtering, must the transactions be successful?
pub const ALLOW_UNSUCCESSFUL_TRANSACTIONS: bool = false;
