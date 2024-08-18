
// Consider splitting this into three separate files, one for each method.
#[derive(Debug, PartialEq)]
pub enum Errors {
    InvalidPrice,
    InvalidFee,
    InvalidLiquidityTarget,
    InvalidDeposit,
    InvalidSwapAmount,
    InvalidLpTokenToRemove,
    ToBigSwap(u64),
}
