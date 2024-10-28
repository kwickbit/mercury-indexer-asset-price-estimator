use zephyr_sdk::soroban_sdk::xdr::{
    AlphaNum12, AlphaNum4, Asset, ClaimAtom, ClaimLiquidityAtom, ClaimOfferAtom, ClaimOfferAtomV0,
    OperationResult, OperationResultTr, PathPaymentStrictReceiveResult,
    PathPaymentStrictReceiveResultSuccess, PathPaymentStrictSendResult,
    PathPaymentStrictSendResultSuccess, TransactionResultMeta, TransactionResultResult,
};

use crate::constants::{scam_addresses::SCAM_ADDRESSES, soroswap_tokens::SOROSWAP_TOKENS};

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

pub fn extract_claim_atoms_from_path_payment_result(
    path_payment_result: &OperationResultTr,
) -> Vec<ClaimAtom> {
    match path_payment_result {
        OperationResultTr::PathPaymentStrictReceive(PathPaymentStrictReceiveResult::Success(
            PathPaymentStrictReceiveResultSuccess { offers, .. },
        ))
        | OperationResultTr::PathPaymentStrictSend(PathPaymentStrictSendResult::Success(
            PathPaymentStrictSendResultSuccess { offers, .. },
        )) => offers.to_vec(),
        _ => unreachable!(),
    }
}

pub fn extract_claim_atom_data(claim_atom: &ClaimAtom) -> (&Asset, i64, &Asset, i64) {
    match claim_atom {
        ClaimAtom::V0(ClaimOfferAtomV0 {
            asset_sold,
            amount_sold,
            asset_bought,
            amount_bought,
            ..
        })
        | ClaimAtom::OrderBook(ClaimOfferAtom {
            asset_sold,
            amount_sold,
            asset_bought,
            amount_bought,
            ..
        })
        | ClaimAtom::LiquidityPool(ClaimLiquidityAtom {
            asset_sold,
            amount_sold,
            asset_bought,
            amount_bought,
            ..
        }) => (asset_sold, *amount_sold, asset_bought, *amount_bought),
    }
}

pub fn format_asset_code(asset: &Asset) -> String {
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

pub fn format_asset_issuer(asset: &Asset) -> String {
    match asset {
        Asset::Native => "Native".to_string(),
        Asset::CreditAlphanum4(AlphaNum4 { issuer, .. })
        | Asset::CreditAlphanum12(AlphaNum12 { issuer, .. }) => issuer.to_string(),
    }
}

pub fn is_floating_asset_valid(asset: &Asset) -> bool {
    let is_scam_address = SCAM_ADDRESSES.contains(&format_asset_code(asset).as_str());

    let is_fake_xlm = match asset {
        Asset::Native => false,
        _ => format_asset_code(asset) == "XLM",
    };

    !is_scam_address && !is_fake_xlm
}

pub fn is_certified_asset(floatcode: &str, fltissuer: &str) -> bool {
    fltissuer == "Native" || SOROSWAP_TOKENS.contains(&(floatcode, fltissuer))
}
