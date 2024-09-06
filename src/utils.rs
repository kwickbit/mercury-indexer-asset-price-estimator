use zephyr_sdk::soroban_sdk::xdr::{Operation, TransactionEnvelope, VecM};

pub const ASSET: &str = "USDC";

pub fn transaction_operations(transaction: &TransactionEnvelope) -> VecM<Operation, 100> {
    match transaction {
        TransactionEnvelope::TxV0(envelope) => envelope.tx.operations.clone(),
        TransactionEnvelope::Tx(envelope) => envelope.tx.operations.clone(),
        _ => Default::default(),
    }
}
