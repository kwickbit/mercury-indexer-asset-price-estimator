use stellar_strkey::{Contract, Strkey};

use time::{format_description::well_known::Iso8601, OffsetDateTime};
use zephyr_sdk::soroban_sdk::xdr::{
    ClaimAtom, ClaimLiquidityAtom, ClaimOfferAtom, ClaimOfferAtomV0, Hash, ManageBuyOfferResult,
    ManageSellOfferResult, OperationResult, OperationResultTr, PathPaymentStrictReceiveResult,
    PathPaymentStrictReceiveResultSuccess, PathPaymentStrictSendResult,
    PathPaymentStrictSendResultSuccess, ScAddress, ScMap, ScVal, ScVec, TransactionResultMeta,
    TransactionResultResult,
};

use crate::{
    config::{scam_addresses::SCAM_ADDRESSES, soroswap_tokens::SOROSWAP_TOKENS, XLM_ADDRESS},
    db::swap::{SwapAsset, SwapData},
};

/**
 *  Extracts the successful transaction results from a TransactionResultMeta.
 *  This is tedious. Oh, and we discard any unsuccessful results.
 */
pub(crate) fn extract_transaction_results(
    result_meta: &TransactionResultMeta,
) -> Vec<OperationResultTr> {
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

/**
 * We build swaps from ClaimAtoms, which represent actual exchanges of assets.
 * We can get these from Offer and PathPayment operations, in a subtly
 * different way.
 */
pub(crate) fn get_claims_from_operation(path_payment_result: &OperationResultTr) -> Vec<ClaimAtom> {
    match path_payment_result {
        OperationResultTr::ManageSellOffer(ManageSellOfferResult::Success(result))
        | OperationResultTr::CreatePassiveSellOffer(ManageSellOfferResult::Success(result))
        | OperationResultTr::ManageBuyOffer(ManageBuyOfferResult::Success(result)) => {
            result.offers_claimed.to_vec()
        }

        OperationResultTr::PathPaymentStrictReceive(PathPaymentStrictReceiveResult::Success(
            PathPaymentStrictReceiveResultSuccess { offers, .. },
        ))
        | OperationResultTr::PathPaymentStrictSend(PathPaymentStrictSendResult::Success(
            PathPaymentStrictSendResultSuccess { offers, .. },
        )) => offers.to_vec(),
        _ => Vec::new(),
    }
}

/**
 * We extract only the data we need from the various types of ClaimAtoms.
 */
pub(crate) fn extract_claim_atom_data(claim_atom: &ClaimAtom) -> SwapData {
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
        }) => SwapData {
            asset_sold: SwapAsset::try_from(asset_sold).ok(),
            amount_sold: *amount_sold,
            asset_bought: SwapAsset::try_from(asset_bought).ok(),
            amount_bought: *amount_bought,
        },
    }
}

/**
 * We highlight Soroswap-certified assets, as well as native XLM.
 */
pub(crate) fn is_certified_asset(floatcode: &str, fltissuer: &str) -> bool {
    fltissuer == "Native"
        || SOROSWAP_TOKENS
            .iter()
            .any(|asset| asset.code == floatcode && asset.issuer == fltissuer)
}

/**
 * Build a SwapAsset from a non-native asset code and issuer.
 */
pub(crate) fn build_nonnative_swap_asset(
    asset_code: &[u8],
    issuer: String,
) -> Result<&'static SwapAsset, String> {
    if SCAM_ADDRESSES.contains(&issuer.as_str()) {
        return Err("Scam asset".to_string());
    }

    let code = format_nonnative_asset(asset_code);

    if code == "XLM" {
        return Err("Non-native XLM asset".to_string());
    }

    SOROSWAP_TOKENS
        .iter()
        .find(|token| token.code == code && token.issuer == issuer)
        .ok_or_else(|| "Asset not found in Soroswap tokens".to_string())
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

/**
 * Convert a Hash to a StrKey.
 */
pub(crate) fn hash_to_strkey(hash: &Hash) -> String {
    // Convert Hash to Contract
    let contract = Contract(hash.0);

    // Convert to StrKey string
    Strkey::Contract(contract).to_string()
}

/**
 * Given a contract address, return a SwapAsset with the asset code and issuer.
 */
pub(crate) fn get_swap_asset(contract_address: String) -> Option<&'static SwapAsset> {
    if contract_address == XLM_ADDRESS {
        Some(&SwapAsset {
            code: "XLM",
            issuer: "Native",
            contract: XLM_ADDRESS,
        })
    } else {
        SOROSWAP_TOKENS
            .iter()
            .find(|asset| asset.contract == contract_address)
    }
}

/**
 * Given a ScVal, return the contract address if it is a contract address.
 */
pub(crate) fn get_address_from_scval(val: &ScVal) -> Option<String> {
    match val {
        ScVal::Address(ScAddress::Contract(contract)) => Some(hash_to_strkey(contract)),
        _ => None,
    }
}

/**
 * Given a ScMap, return the value of a given key if it exists.
 */
pub(crate) fn scmap_get(map: &ScMap, key: String) -> Option<&ScVec> {
    map.0
        .iter()
        .find(|entry| {
            matches!(
                &entry.key,
                ScVal::Symbol(s) if s.to_string() == key
            )
        })
        .and_then(|entry| match &entry.val {
            ScVal::Vec(Some(value)) => Some(value),
            _ => None,
        })
}

/**
 * Return the string representation of a timestamp.
 */
pub(crate) fn parse_date(timestamp: &i64) -> String {
    OffsetDateTime::from_unix_timestamp(*timestamp)
        .unwrap()
        .format(&Iso8601::DEFAULT)
        .unwrap()
}
