use zephyr_sdk::soroban_sdk::xdr::{
    Operation, OperationResult, TransactionEnvelope, TransactionResultMeta, TransactionResultResult,
};

use crate::utils;

#[derive(Debug)]
pub struct InterestingTransaction {
    pub hash: String,
    pub operations: Vec<Operation>,
    pub results: Vec<OperationResult>,
}

impl InterestingTransaction {
    pub fn new(envelope: &TransactionEnvelope, result_meta: &TransactionResultMeta) -> Self {
        let operations = utils::extract_transaction_operations(envelope);
        let results = match &result_meta.result.result.result {
            TransactionResultResult::TxSuccess(success) => success.to_vec(),
            _ => unreachable!(),
        };
        Self {
            hash: result_meta.result.transaction_hash.to_string(),
            operations: operations.to_vec(),
            results,
        }
    }
}
