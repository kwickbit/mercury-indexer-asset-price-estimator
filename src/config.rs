// The stablecoins we focus on
pub const STABLECOINS: [&str; 3] = ["USDC", "USDT", "DAI"];

// Amounts are represented with this factor
pub const CONVERSION_FACTOR: f64 = 10_000_000.0;

// Length of the exchange rate window in seconds
pub const RATE_UPDATE_INTERVAL: u64 = 900;
