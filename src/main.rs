use crate::calc::{divide, multiply};
use crate::errors::Errors;

mod errors;
mod calc;


#[derive(Debug)]
struct TokenAmount(u64);
#[derive(Debug)]
struct StakedTokenAmount(u64);
struct LpTokenAmount(u64);
struct Price(u64);
struct Percentage(u64);

struct LpPool {
    price: Price,
    token_amount_reserve: TokenAmount,
    st_token_amount: StakedTokenAmount,
    lp_token_amount: LpTokenAmount,
    liquidity_target: TokenAmount,
    min_fee: Percentage,
    max_fee: Percentage,
}

impl LpPool {
    pub fn init(
        price: Price,
        min_fee: Percentage,
        max_fee: Percentage,
        liquidity_target: TokenAmount
    ) -> Result<Self, Errors> {

        if min_fee.0 > max_fee.0 {
            return Err(Errors::InvalidFee)
        }

        if price.0 == 0{
            return Err(Errors::InvalidPrice)
        }

        if liquidity_target.0 == 0 {
            return Err(Errors::InvalidLiquidityTarget)
        }

        Ok(LpPool {
            price,
            token_amount_reserve: TokenAmount(0),
            st_token_amount: StakedTokenAmount(0),
            lp_token_amount: LpTokenAmount(0),
            liquidity_target,
            min_fee,
            max_fee,
        })
    }

    pub fn add_liquidity(
        self: &mut Self,
        token_amount: TokenAmount
    ) -> Result<LpTokenAmount, Errors>{
        if token_amount.0 == 0{
            return Err(Errors::InvalidDeposit)
        }


        if self.token_amount_reserve.0 == 0 {
            self.token_amount_reserve.0 += token_amount.0;
            self.lp_token_amount.0 += token_amount.0;
            return Ok(LpTokenAmount(token_amount.0))
        }


        let sum_token = self.token_amount_reserve.0  + multiply(self.st_token_amount.0, self.price.0);
        let new_token_amount = sum_token + token_amount.0;
        let token_propotion = divide(new_token_amount, sum_token);
        let lp_token_minted = multiply(self.lp_token_amount.0, token_propotion) - self.lp_token_amount.0;

        self.token_amount_reserve.0 += token_amount.0;
        self.lp_token_amount.0 += lp_token_minted;

        Ok(LpTokenAmount(lp_token_minted))
    }

    pub fn remove_liquidity(
        self: &mut Self,
        lp_token_amount: LpTokenAmount
    ) -> Result<(TokenAmount,StakedTokenAmount), Errors>{
        if lp_token_amount.0 == 0 || lp_token_amount.0 > self.lp_token_amount.0 {
            return Err(Errors::InvalidLpTokenToRemove)
        }

        let part_of_lp = divide(lp_token_amount.0, self.lp_token_amount.0);

        let token_to_receive = multiply(part_of_lp, self.token_amount_reserve.0);
        self.token_amount_reserve.0 -= token_to_receive;

        let st_token_to_receive = multiply(part_of_lp, self.st_token_amount.0);
        self.st_token_amount.0 -= st_token_to_receive;

        Ok((TokenAmount(token_to_receive), StakedTokenAmount(st_token_to_receive)))

    }

    pub fn swap(self: &mut Self,
                staked_token_amount: StakedTokenAmount
    ) -> Result<TokenAmount, Errors>{
        if staked_token_amount.0 == 0{
            return Err(Errors::InvalidSwapAmount)
        }

        let token_for_st = multiply(self.price.0, staked_token_amount.0);
        let token_min_fee = token_for_st - multiply(token_for_st, self.min_fee.0);

        // IF REVERVE < 0
        let token_receive_max_fee = token_for_st - multiply(token_for_st, self.max_fee.0);

        if token_receive_max_fee > self.token_amount_reserve.0{
            let token_max_fee = self.token_amount_reserve.0 - multiply(self.token_amount_reserve.0, self.max_fee.0);
            let st_max_swap = divide(token_max_fee, self.price.0);
            return Err(Errors::ToBigSwap(st_max_swap))
        }


        let check_target = self.token_amount_reserve.0 - token_min_fee;


        if check_target > self.liquidity_target.0 || self.min_fee.0 == self.max_fee.0 || self.liquidity_target.0 == 0{
            self.st_token_amount.0 += staked_token_amount.0;
            self.token_amount_reserve.0 -= token_min_fee;
            return Ok(TokenAmount(token_min_fee))
        }



        let diff_token = self.token_amount_reserve.0 - token_for_st; // simple diffrence without fee

        let target_part = divide(diff_token, self.liquidity_target.0);
        let calc_fee = self.max_fee.0 - multiply(self.max_fee.0 - self.min_fee.0, target_part);
        let token_to_receive = token_for_st - multiply(token_for_st ,calc_fee);
        self.token_amount_reserve.0 -= token_to_receive;
        self.st_token_amount.0 += staked_token_amount.0;
        Ok(TokenAmount(token_to_receive))
    }
}


fn main() {
    let mut lp = Result::unwrap(LpPool::init(Price(1_500_000), Percentage(1_000), Percentage(90_000), TokenAmount(90_000_000)));
    let a = LpPool::add_liquidity(&mut lp, TokenAmount(100*1_000_000)).unwrap().0;

    println!("Hello, {}", a.clone());
    println!("Hello, {}", LpPool::swap(&mut lp, StakedTokenAmount(6*1_000_000)).unwrap().0 );
    let b = LpPool::add_liquidity(&mut lp, TokenAmount(10*1_000_000)).unwrap().0;
    println!("Hello, {}", b.clone() );
    println!("Hello, {}", LpPool::swap(&mut lp, StakedTokenAmount(30*1_000_000)).unwrap().0 );
    let c = a+b;
    let (token_amount, staked_token_amount) = LpPool::remove_liquidity(&mut lp, LpTokenAmount(c)).unwrap();
    println!("Token Amount: {:?}", token_amount);
    println!("Staked Token Amount: {:?}", staked_token_amount);

}


