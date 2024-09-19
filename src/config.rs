// Do I want milestones to be printed even when there are transactions?
pub const FORCE_MILESTONE: bool = true;

// How often should milestones be printed?
// There are 11-12 sequences every minute.
pub const MILESTONE_INTERVAL: u32 = 12;

// Whether to print details of interesting transactions
// We always print at least the number of interesting transactions.
pub const PRINT_TRANSACTION_DETAILS: bool = true;

// Whether to interact with the DB at all
pub const DO_DB_STUFF: bool = false;

// When filtering, must the transactions be successful?
pub const ALLOW_UNSUCCESSFUL_TRANSACTIONS: bool = false;

// The stablecoins we focus on
pub const STABLECOINS: [&str; 3] = ["USDC", "USDT", "DAI"];
