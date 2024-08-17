

#[derive(Debug)]
pub enum Errors {
    InvalidPrice,
    InvalidFee,
    InvalidLiquidityTarget,
    InvalidDeposit,
    InvalidSwapAmount,
    InvalidLpTokenToRemove,
    ToBigSwap(u64),
}