mod api;
mod config;
mod db;
mod filter;
mod utils;

use config::{CONVERSION_FACTOR, USDC};
use db::swap::Swap;
use utils::format_asset;
use zephyr_sdk::{
    soroban_sdk::xdr::{
        Asset, ClaimAtom, ClaimLiquidityAtom, ClaimOfferAtom, ClaimOfferAtomV0, OperationResultTr,
        PathPaymentStrictReceiveResult, PathPaymentStrictReceiveResultSuccess,
        PathPaymentStrictSendResult, PathPaymentStrictSendResultSuccess,
    },
    EnvClient,
};

#[no_mangle]
pub extern "C" fn on_close() {
    // The basics
    let client = EnvClient::new();

    // Read and save the swaps from the latest sequence
    let results = client.reader().tx_processing();
    let path_payment_results = filter::path_payment_results(&results);

    client.log().debug(
        &format!("Sequence {}", client.reader().ledger_sequence(),),
        None,
    );

    let swaps_from_path_payment_results = path_payment_results
        .iter()
        .filter_map(usdc_ppr_swap)
        .collect::<Vec<Swap>>();

    client.log().debug(
        &format!("Path payment swaps: {:#?}", swaps_from_path_payment_results),
        None,
    );

    let swaps_from_offer_results = filter::swaps(results);

    let all_swaps = [
        &swaps_from_path_payment_results[..],
        &swaps_from_offer_results[..],
    ]
    .concat();

    db::save_swaps(&client, &all_swaps);

    // If it is time, calculate and save the exchange rates from the latest sequence
    db::save_rates(&client);
}

fn usdc_ppr_swap(path_payment_result: &OperationResultTr) -> Option<Swap> {
    let offers = extract_offers(path_payment_result);

    offers.iter().fold(None, |accumulator, claim_atom| {
        let (asset_sold, amount_sold, asset_bought, amount_bought) =
            extract_claim_atom_data(claim_atom);

        let stablecoin = "USDC".to_string();

        if *asset_sold == USDC {
            Some(Swap {
                created_at: None,
                stablecoin,
                stablecoin_amount: amount_sold as f64,
                is_stablecoin_sale: true,
                floating_asset: format_asset(asset_bought),
                price_numerator: amount_bought.try_into().unwrap(),
                price_denominator: amount_sold.try_into().unwrap(),
            })
        } else if *asset_bought == USDC {
            Some(Swap {
                created_at: None,
                stablecoin,
                stablecoin_amount: amount_bought as f64,
                is_stablecoin_sale: false,
                floating_asset: format_asset(asset_sold),
                price_numerator: amount_sold.try_into().unwrap(),
                price_denominator: amount_bought.try_into().unwrap(),
            })
        } else {
            accumulator
        }
    })
}

fn print_pprs(path_payment_results: &[OperationResultTr]) -> String {
    path_payment_results
        .iter()
        .map(print_ppr)
        .collect::<Vec<String>>()
        .join(" // ")
}

fn extract_offers(ppr: &OperationResultTr) -> Vec<ClaimAtom> {
    match ppr {
        OperationResultTr::PathPaymentStrictReceive(PathPaymentStrictReceiveResult::Success(
            PathPaymentStrictReceiveResultSuccess { offers, .. },
        ))
        | OperationResultTr::PathPaymentStrictSend(PathPaymentStrictSendResult::Success(
            PathPaymentStrictSendResultSuccess { offers, .. },
        )) => offers.to_vec(),
        _ => unreachable!(),
    }
}

fn print_ppr(path_payment_result: &OperationResultTr) -> String {
    let offers = extract_offers(path_payment_result);

    let operation_type = if let OperationResultTr::PathPaymentStrictReceive(_) = path_payment_result
    {
        "Receive"
    } else {
        "Send"
    };

    let formatted_offers = offers
        .iter()
        .enumerate()
        .map(print_claim_atom)
        .collect::<Vec<String>>()
        .join("; ");

    format!("#### {operation_type} result. Offers: {formatted_offers}")
}

fn extract_claim_atom_data(claim_atom: &ClaimAtom) -> (&Asset, i64, &Asset, i64) {
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
        _ => unreachable!(),
    }
}

fn print_claim_atom((count, claim_atom): (usize, &ClaimAtom)) -> String {
    let (asset_sold, amount_sold, asset_bought, amount_bought) =
        extract_claim_atom_data(claim_atom);

    format!(
        "Claim {}: {} {} => {} {}",
        count + 1,
        amount_bought as f64 / CONVERSION_FACTOR,
        format_asset(asset_bought),
        amount_sold as f64 / CONVERSION_FACTOR,
        format_asset(asset_sold),
    )
}
