#[derive(Debug)]
pub struct Swap {
    pub stablecoin: String,
    pub stablecoin_amount: i64,
    pub floating_asset: String,
    pub price_numerator: i32,
    pub price_denominator: i32,
}
