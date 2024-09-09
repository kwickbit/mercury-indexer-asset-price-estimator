use zephyr_sdk::soroban_sdk::xdr::{Operation, TransactionEnvelope, VecM};

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
