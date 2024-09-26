// The stablecoins we focus on
pub const STABLECOINS: [&str; 3] = ["USDC", "USDT", "DAI"];

// Amounts are represented with this factor
pub const CONVERSION_FACTOR: f64 = 10_000_000.0;

/*
The following constants are mostly for convenience.
These things were changing very quickly during early development,
so it made sense to group them here.
*/

// How often do we log exchange rates, measured in sequences
// There are 11-12 sequences per minute
pub const LOGGING_INTERVAL: u32 = 25;

// Do we save the swaps to the database?
pub const SAVE_SWAPS_TO_DATABASE: bool = true;

// Do we save the exchange rates to the database?
pub const SAVE_RATES_TO_DATABASE: bool = true;

// How often do we update the savepoint in seconds?
pub const RATE_UPDATE_INTERVAL: u64 = 300;
