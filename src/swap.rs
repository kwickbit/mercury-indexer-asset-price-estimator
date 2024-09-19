#[derive(Debug)]
pub struct Swap {
    pub stablecoin: String,
    pub stablecoin_amount: i64,
    pub floating_asset: String,
    pub price_numerator: i32,
    pub price_denominator: i32,
}

pub fn format_swap(swap: &Swap) -> String {
    format!(
        "{} {} for {} at {}",
        swap.stablecoin_amount as f64 / 10_000_000.0,
        swap.stablecoin,
        swap.floating_asset,
        swap.price_numerator as f32 / swap.price_denominator as f32,
    )
}
