use zephyr_sdk::soroban_sdk::xdr::{AccountId, AlphaNum4, Asset, AssetCode4, PublicKey, Uint256};

// We focus on USDC, the most-used stablecoin on the Stellar network.
pub const USDC: Asset = Asset::CreditAlphanum4(AlphaNum4 {
    asset_code: AssetCode4(*b"USDC"),
    issuer: AccountId(PublicKey::PublicKeyTypeEd25519(Uint256([
        59, 153, 17, 56, 14, 254, 152, 139, 160, 168, 144, 14, 177, 207, 228, 79, 54, 111, 125,
        190, 148, 107, 237, 7, 114, 64, 247, 246, 36, 223, 21, 197,
    ]))),
});

// Amounts are represented multiplied by this factor
pub const CONVERSION_FACTOR: f64 = 10_000_000.0;

// Length of the exchange rate window
const MINUTE: u64 = 60;
const _HOUR: u64 = 60 * MINUTE;
const _DAY: u64 = 24 * _HOUR;
pub const RATE_UPDATE_INTERVAL: u64 = 60 * MINUTE;
