// The stablecoins we focus on
pub const STABLECOINS: [&str; 3] = ["USDC", "USDT", "DAI"];

// Amounts are represented with this factor
pub const CONVERSION_FACTOR: f64 = 10_000_000.0;

// Length of the exchange rate window
const MINUTE: u64 = 60;
const _HOUR: u64 = 60 * MINUTE;
const _DAY: u64 = 24 * _HOUR;
pub const RATE_UPDATE_INTERVAL: u64 = 2 * MINUTE;
