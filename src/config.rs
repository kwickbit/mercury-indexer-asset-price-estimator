// The stablecoins we focus on
pub const STABLECOINS: [&str; 3] = ["USDC", "USDT", "DAI"];

// Amounts are represented with this factor
pub const CONVERSION_FACTOR: f64 = 10_000_000.0;

/*
The following constants are mostly for convenience.
These things were changing very quickly during early development,
so it made sense to group them here.
*/

// Do we save the swaps to the database?
pub const SAVE_SWAPS_TO_DATABASE: bool = false;
