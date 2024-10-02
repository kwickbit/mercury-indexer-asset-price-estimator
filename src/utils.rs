use zephyr_sdk::soroban_sdk::xdr::{
    AlphaNum12, AlphaNum4, Asset, OperationBody, OperationResult, OperationResultTr, TransactionEnvelope, TransactionResultMeta, TransactionResultResult
};

#[allow(dead_code)]
pub fn extract_transaction_operations(transaction: &TransactionEnvelope) -> Vec<OperationBody> {
    let operations = match transaction {
        TransactionEnvelope::TxV0(envelope) => &envelope.tx.operations,
        TransactionEnvelope::Tx(envelope) => &envelope.tx.operations,
        _ => &Default::default(),
    };

    operations.iter().map(|op| op.body.clone()).collect()
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

fn bytes_to_string(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).unwrap_or("Unreadable")
}
