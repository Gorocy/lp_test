
const PRECISION: u64 = 1_000_000;


pub fn multiply(a: u64, b: u64) -> u64{
    let result = a * b;
    result/PRECISION
}

pub fn divide(a: u64, b: u64) -> u64{
    let a_scale = a*PRECISION;
    let result = a_scale/b;
    result
}
