use zephyr_sdk::soroban_sdk::xdr::{
    AlphaNum12, AlphaNum4, Asset, Operation, OperationBody, OperationResult, OperationResultTr,
    TransactionEnvelope, TransactionResultMeta, TransactionResultResult, VecM,
};

use crate::config::STABLECOINS;

#[allow(dead_code)]
pub fn extract_transaction_operations_with_results(
    event: &(&TransactionEnvelope, &TransactionResultMeta),
) -> Vec<(OperationBody, OperationResultTr)> {
    let &(envelope, result_meta) = event;
    let operations = extract_transaction_operations(envelope);
    let results = extract_transaction_results(result_meta);

    operations
        .to_vec()
        .into_iter()
        .zip(results.into_iter())
        .map(|(op, res)| (op.body, res))
        .collect()
}

pub fn extract_transaction_operations(transaction: &TransactionEnvelope) -> VecM<Operation, 100> {
    match transaction {
        TransactionEnvelope::TxV0(envelope) => envelope.tx.operations.clone(),
        TransactionEnvelope::Tx(envelope) => envelope.tx.operations.clone(),
        _ => Default::default(),
    }
}

pub fn extract_transaction_results(result_meta: &TransactionResultMeta) -> Vec<OperationResultTr> {
    match &result_meta.result.result.result {
        TransactionResultResult::TxSuccess(op_results) => op_results
            .iter()
            .filter_map(|op_result| match op_result {
                OperationResult::OpInner(inner_op_result) => Some(inner_op_result.clone()),
                _ => None,
            })
            .collect(),
        _ => Default::default(),
    }
}

pub fn bytes_to_string(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).unwrap_or("Unreadable")
}

pub fn is_stablecoin(asset: &Asset) -> bool {
    match asset {
        Asset::Native => false,
        Asset::CreditAlphanum4(AlphaNum4 { asset_code, .. }) => {
            compare_asset_code(asset_code.as_slice())
        }
        Asset::CreditAlphanum12(AlphaNum12 { asset_code, .. }) => {
            compare_asset_code(asset_code.as_slice())
        }
    }
}

fn compare_asset_code(code: &[u8]) -> bool {
    match std::str::from_utf8(code) {
        Ok(str) => STABLECOINS.contains(&str.trim_end_matches('\0')),
        Err(_) => false,
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
