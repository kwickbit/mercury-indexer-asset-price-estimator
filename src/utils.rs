use zephyr_sdk::soroban_sdk::xdr::{
    AlphaNum12, AlphaNum4, Asset, Operation, TransactionEnvelope, VecM,
};

pub const ASSET: &str = "USDC";

pub fn extract_transaction_operations(transaction: &TransactionEnvelope) -> VecM<Operation, 100> {
    match transaction {
        TransactionEnvelope::TxV0(envelope) => envelope.tx.operations.clone(),
        TransactionEnvelope::Tx(envelope) => envelope.tx.operations.clone(),
        _ => Default::default(),
    }
}

pub fn bytes_to_string(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(alpha) => alpha.to_string(),
        Err(_) => "Unreadable".to_string(),
    }
}

pub fn asset_matches(asset: &AlphaNum4, code: &str) -> bool {
    asset.asset_code.as_slice() == code.as_bytes()
}

pub fn asset_is_usdc(asset: &Asset) -> bool {
    match asset {
        Asset::CreditAlphanum4(inner_asset) => asset_matches(inner_asset, ASSET),
        _ => false,
    }
}

pub fn format_asset(asset: &Asset) -> String {
    match asset {
        Asset::Native => "XLM".to_string(),
        Asset::CreditAlphanum4(AlphaNum4 { asset_code, .. }) => {
            format_nonnative_asset(asset_code.as_slice())
        }
        Asset::CreditAlphanum12(AlphaNum12 { asset_code, .. }) => {
            format_nonnative_asset(asset_code.as_slice())
        }
    }
}

fn format_nonnative_asset(asset_code: &[u8]) -> String {
    bytes_to_string(asset_code)
        .chars()
        .filter(|char| char.is_ascii_alphabetic())
        .collect()
}
