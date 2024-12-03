pub(crate) mod scam_addresses;
pub(crate) mod soroswap_tokens;

use crate::db::swap::SwapAsset;

// We focus on USDC, the most-used stablecoin on the Stellar network.
pub const USDC: SwapAsset = SwapAsset {
    code: "USDC",
    issuer: "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN",
    contract: "CCW67TSZV3SSS2HXMBQ5JFGCKJNXKZM7UQUWUZPUTHXSTZLEO7SJMI75",
};

// We need to know the SoroswapRouter contract address to filter Soroswap swaps
pub const SOROSWAP_ROUTER: &str = "CAG5LRYQ5JVEUI5TEID72EYOVX44TTUJT5BQR2J6J77FH65PCCFAJDDH";

// Amounts are represented multiplied by this factor
pub const CONVERSION_FACTOR: f64 = 10_000_000.0;

// Length of the exchange rate window
const MINUTE: u64 = 60;
const _HOUR: u64 = 60 * MINUTE;
const _DAY: u64 = 24 * _HOUR;
pub const RATE_UPDATE_INTERVAL: u64 = 60 * MINUTE;
