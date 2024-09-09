use zephyr_sdk::soroban_sdk::xdr::{
    Operation, TransactionEnvelope, TransactionResultMeta, TransactionResultResult,
};

use crate::utils;

pub struct InterestingTransaction<'a> {
    #[allow(dead_code)]
    pub envelope: &'a TransactionEnvelope,
    pub operations: Vec<Operation>,
    pub result: &'a TransactionResultResult,
}

impl<'a> InterestingTransaction<'a> {
    pub fn new(envelope: &'a TransactionEnvelope, result_meta: &'a TransactionResultMeta) -> Self {
        let operations = utils::extract_transaction_operations(envelope);
        Self {
            envelope,
            operations: operations.to_vec(),
            result: &result_meta.result.result.result,
        }
    }

    pub fn is_successful(&self) -> bool {
        matches!(self.result, TransactionResultResult::TxSuccess(_))
    }
}
