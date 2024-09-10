use zephyr_sdk::soroban_sdk::xdr::{
    Operation, OperationResult, TransactionEnvelope, TransactionResultMeta, TransactionResultResult,
};

use crate::exchange_rate::{extract_exchange_rates, ExchangeRate};
use crate::utils;

#[derive(Debug)]
pub struct InterestingTransaction {
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
            operations: operations.to_vec(),
            results,
        }
    }

    pub fn exchange_rates(&self) -> Vec<ExchangeRate> {
        self.operations
            .iter()
            .zip(self.results.iter())
            .filter_map(|(op, result)| extract_exchange_rates(op, result))
            .flatten()
            .collect()
    }
}
