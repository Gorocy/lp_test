// 1_000_000 standard for most common tokens on Solana
// even Solana have 1_000_000_000
const PRECISION: u64 = 1_000_000;


// Personally, I would round the fee in favor of the liquidity pool.
// However, when I started rounding, I noticed that it would not pass the historical tests. :)
pub fn multiply(a: u64, b: u64) -> u64 {
    let result = a * b;
    result / PRECISION
}

pub fn divide(a: u64, b: u64) -> u64 {
    let a_scale = a * PRECISION;
    let result = a_scale / b;
    result
}
