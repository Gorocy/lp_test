use crate::calc::{divide, multiply};
use crate::errors::Errors;

mod calc;
mod errors;

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
        liquidity_target: TokenAmount,
    ) -> Result<Self, Errors> {
        // min_fee have to be smaller or equal, but I assume in this model only smaller
        if min_fee.0 >= max_fee.0 {
            return Err(Errors::InvalidFee);
        }
        // price grater that 0
        if price.0 == 0 {
            return Err(Errors::InvalidPrice);
        }
        // target should be greater than 0
        // if target == 0, fee would be a constant
        if liquidity_target.0 == 0 {
            return Err(Errors::InvalidLiquidityTarget);
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
        token_amount: TokenAmount,
    ) -> Result<LpTokenAmount, Errors> {
        // deposit must be grater than 0
        if token_amount.0 == 0 {
            return Err(Errors::InvalidDeposit);
        }

        // if token == lp_token and st_token == 0, then
        // calculating is unnecessary
        if self.lp_token_amount.0 == self.token_amount_reserve.0 && self.st_token_amount.0 == 0 {
            self.token_amount_reserve.0 += token_amount.0;
            self.lp_token_amount.0 += token_amount.0;
            return Ok(LpTokenAmount(token_amount.0));
        }

        // value of tokens and st_tokens in tokens before deposit 
        let value_token =
            self.token_amount_reserve.0 + multiply(self.st_token_amount.0, self.price.0);
        // value of tokens and st_tokens in tokens after deposit 
        let new_value_token = value_token + token_amount.0;
        // proportion after to before
        let token_proportion = divide(new_value_token, value_token);
        // we need to receive the same proportion of lp_tokens before to lp_tokens after
        let lp_token_minted =
            multiply(self.lp_token_amount.0, token_proportion) - self.lp_token_amount.0;
        
        // deposit tokens
        self.token_amount_reserve.0 += token_amount.0;
        // saving information about lp_tokens existing
        self.lp_token_amount.0 += lp_token_minted;

        Ok(LpTokenAmount(lp_token_minted))
    }

    pub fn remove_liquidity(
        self: &mut Self,
        lp_token_amount: LpTokenAmount,
    ) -> Result<(TokenAmount, StakedTokenAmount), Errors> {
        // cannot withdraw for 0 or more than exist(it shouldn't happen ever, but it is simulator)
        if lp_token_amount.0 == 0 || lp_token_amount.0 > self.lp_token_amount.0 {
            return Err(Errors::InvalidLpTokenToRemove)
        }
        // what part send of existing
        let part_of_lp = divide(lp_token_amount.0, self.lp_token_amount.0);

        // update existing amount
        self.lp_token_amount.0 -= lp_token_amount.0;

        // calculating proportion for token and update amount in pool
        let token_to_receive = multiply(part_of_lp, self.token_amount_reserve.0);
        self.token_amount_reserve.0 -= token_to_receive;

        // calculating proportion for st_token and update amount in pool
        let st_token_to_receive = multiply(part_of_lp, self.st_token_amount.0);
        self.st_token_amount.0 -= st_token_to_receive;

        Ok((
            TokenAmount(token_to_receive),
            StakedTokenAmount(st_token_to_receive),
        ))
    }

    pub fn swap(
        self: &mut Self,
        staked_token_amount: StakedTokenAmount,
    ) -> Result<TokenAmount, Errors> {
        // cannot swap 0
        if staked_token_amount.0 == 0 {
            return Err(Errors::InvalidSwapAmount);
        }

        // token for send st_token amount
        let token_for_st = multiply(self.price.0, staked_token_amount.0);

        // check amount after max_fee
        let token_receive_max_fee = token_for_st - multiply(token_for_st, self.max_fee.0);

        // cannot swap if pool don't have enough
        if token_receive_max_fee > self.token_amount_reserve.0 {
            // check biggest possible amount for token with max_fee
            let token_max_fee =
                self.token_amount_reserve.0 - multiply(self.token_amount_reserve.0, self.max_fee.0);
            let st_max_swap = divide(token_max_fee, self.price.0);
            return Err(Errors::ToBigSwap(st_max_swap));
        }

        // check amount after min_fee
        let token_min_fee = token_for_st - multiply(token_for_st, self.min_fee.0);
        let check_target = self.token_amount_reserve.0 - token_min_fee;

        // there will be more tokens left in the pool than target
        if check_target > self.liquidity_target.0 || self.min_fee.0 == self.max_fee.0 {
            self.st_token_amount.0 += staked_token_amount.0;
            self.token_amount_reserve.0 -= token_min_fee;
            return Ok(TokenAmount(token_min_fee));
        }

        // amount will be,
        // maybe it should be checked with token_for_st - min_fee,
        // but it would pass history
        let amount_after = self.token_amount_reserve.0 - token_for_st;

        // proportion amount after to target
        let target_part = divide(amount_after, self.liquidity_target.0);

        // calculate the unstake fee using the formula:
        // unstake_fee = maxFee - (maxFee - minFee) * amountAfter / target

        // this formula computes the fee based on the difference between the maximum and minimum fee,
        // proportionally adjusted by the ratio of amount after to target.
        let unstake_fee = self.max_fee.0 - multiply(self.max_fee.0 - self.min_fee.0, target_part);

        // tokens user will receive after fee
        let token_to_receive = token_for_st - multiply(token_for_st, unstake_fee);

        // update amount of tokens
        self.token_amount_reserve.0 -= token_to_receive;
        // update amount of st_tokens
        self.st_token_amount.0 += staked_token_amount.0;

        Ok(TokenAmount(token_to_receive))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /*
     STORY
    */
    #[test]
    fn test_lp_pool_initialization() {
        let pool = LpPool::init(Price(1__500_000), Percentage(1_000), Percentage(90_000), TokenAmount(90__000_000)).unwrap();
        assert_eq!(pool.price.0, 1__500_000);
        assert_eq!(pool.token_amount_reserve.0, 0);
        assert_eq!(pool.st_token_amount.0, 0);
        assert_eq!(pool.lp_token_amount.0, 0);
        assert_eq!(pool.liquidity_target.0, 90__000_000);
        assert_eq!(pool.min_fee.0, 1_000);
        assert_eq!(pool.max_fee.0, 90_000);
    }

    #[test]
    fn test_add_liquidity_1() {
        let mut pool = LpPool::init(Price(1__500_000), Percentage(1_000), Percentage(90_000), TokenAmount(90__000_000)).unwrap();
        let minted_lp = pool.add_liquidity(TokenAmount(100_000000)).unwrap();
        assert_eq!(minted_lp.0, 100__000_000);
        assert_eq!(pool.token_amount_reserve.0, 100__000_000);
        assert_eq!(pool.lp_token_amount.0, 100__000_000);
    }

    #[test]
    fn test_swap_1() {
        let mut pool = LpPool::init(Price(1__500000), Percentage(1_000), Percentage(90_000), TokenAmount(90__000_000)).unwrap();
        pool.add_liquidity(TokenAmount(100__000_000)).unwrap();
        let received_token = pool.swap(StakedTokenAmount(6__000_000)).unwrap();

        assert_eq!(received_token.0, 8__991_000);
        assert_eq!(pool.price.0, 1__500_000);
        assert_eq!(pool.token_amount_reserve.0, 91__009_000);
        assert_eq!(pool.st_token_amount.0, 6__000_000);
        assert_eq!(pool.lp_token_amount.0, 100__000_000);
        assert_eq!(pool.liquidity_target.0, 90__000_000);
        assert_eq!(pool.min_fee.0, 1_000);
        assert_eq!(pool.max_fee.0, 90_000);
    }

    #[test]
    fn test_add_liquidity_2() {
        let mut pool = LpPool::init(Price(1__500000), Percentage(1_000), Percentage(90_000), TokenAmount(90__000_000)).unwrap();
        pool.add_liquidity(TokenAmount(100__000_000)).unwrap();
        pool.swap(StakedTokenAmount(6__000_000)).unwrap();
        let minted_lp = pool.add_liquidity(TokenAmount(10__000_000)).unwrap();

        assert_eq!(minted_lp.0, 9__999_100);
        assert_eq!(pool.lp_token_amount.0, 109__999_100);
        // TODO
        assert_eq!(pool.price.0, 1__500_000);
        assert_eq!(pool.token_amount_reserve.0, 101__009_000);
        assert_eq!(pool.st_token_amount.0, 6__000_000);
        assert_eq!(pool.lp_token_amount.0, 109__999_100);
        assert_eq!(pool.liquidity_target.0, 90__000_000);
        assert_eq!(pool.min_fee.0, 1_000);
        assert_eq!(pool.max_fee.0, 90_000);

    }

    #[test]
    fn test_swap_2() {
        let mut pool = LpPool::init(Price(1__500_000), Percentage(1_000), Percentage(90_000), TokenAmount(90__000_000)).unwrap();
        pool.add_liquidity(TokenAmount(100__000_000)).unwrap();
        pool.swap(StakedTokenAmount(6__000_000)).unwrap();
        pool.add_liquidity(TokenAmount(10__000_000)).unwrap();
        let received_token = pool.swap(StakedTokenAmount(30__000_000)).unwrap();

        assert_eq!(received_token.0, 43_442_370);
        assert_eq!(pool.lp_token_amount.0, 109__999_100);
        // TODO
        assert_eq!(pool.price.0, 1__500_000);
        assert_eq!(pool.token_amount_reserve.0, 57__566_630);
        assert_eq!(pool.st_token_amount.0, 36__000_000);
        assert_eq!(pool.lp_token_amount.0, 109_999_100);
        assert_eq!(pool.liquidity_target.0, 90__000_000);
        assert_eq!(pool.min_fee.0, 1_000);
        assert_eq!(pool.max_fee.0, 90_000);

    }

    #[test]
    fn test_remove_liquidity() {

        let mut pool = LpPool::init(Price(1__500_000), Percentage(1_000), Percentage(90_000), TokenAmount(90__000_000)).unwrap();
        pool.add_liquidity(TokenAmount(100__000_000)).unwrap();
        pool.swap(StakedTokenAmount(6__000_000)).unwrap();
        pool.add_liquidity(TokenAmount(10__000_000)).unwrap();
        pool.swap(StakedTokenAmount(30__000_000)).unwrap();

        let (token, staked_token) = pool.remove_liquidity(LpTokenAmount(109__999_100)).unwrap();
        assert_eq!(staked_token.0, 36__000_000);
        assert_eq!(token.0, 57_566_630);
        assert_eq!(pool.price.0, 1__500_000);
        assert_eq!(pool.token_amount_reserve.0, 0);
        assert_eq!(pool.st_token_amount.0, 0);
        assert_eq!(pool.lp_token_amount.0, 0);
        assert_eq!(pool.liquidity_target.0, 90__000_000);
        assert_eq!(pool.min_fee.0, 1_000);
        assert_eq!(pool.max_fee.0, 90_000);
    }


    /*
    END OF STORY
     */


    #[test]
    fn test_it_doesnt_allow_to_remove_too_much_liquidity() {

        let mut pool = LpPool::init(Price(1__500_000), Percentage(1_000), Percentage(90_000), TokenAmount(90__000_000)).unwrap();
        pool.add_liquidity(TokenAmount(100__000_000)).unwrap();
        pool.swap(StakedTokenAmount(6__000_000)).unwrap();
        pool.add_liquidity(TokenAmount(10__000_000)).unwrap();
        pool.swap(StakedTokenAmount(30__000_000)).unwrap();

        let result= pool.remove_liquidity(LpTokenAmount(109__999_101));

        match result {
            Err(Errors::InvalidLpTokenToRemove) => {
                assert!(true, "Error occurred as expected: InvalidLpTokenToRemove");
            }
            Err(e) => {
                panic!("Unexpected error: {:?}", e);
            }
            Ok((token, staked_token)) => {
                panic!("Expected an error but received a successful result: token: {:?}, staked_token: {:?}", token, staked_token);
            }
        }

    }

    #[test]
    fn test_pool_initialization_fail_of_too_small_difference_of_fee() {
        // min_fee = max_fee
        match LpPool::init(Price(1__500_000), Percentage(1_000), Percentage(1_000), TokenAmount(90__000_000)) {
            Err(Errors::InvalidFee) => {
                assert!(true, "Error occurred as expected: InvalidFee");
            }
            Err(e) => {
                panic!("Unexpected error: {:?}", e);
            }
            Ok(_lp_pool) => {
                panic!("Expected an error 'InvalidFee', but received a successful init");
            }
        }
    }

    #[test]
    fn test_pool_initialization_doesnt_allow_of_too_small_price() {
        // too small price
        match LpPool::init(Price(0), Percentage(1_000), Percentage(90_000), TokenAmount(90__000_000)){
            Err(Errors::InvalidPrice) => {
                assert!(true, "Error occurred as expected: InvalidPrice");
            }
            Err(e) => {
                panic!("Unexpected error: {:?}", e);
            }
            Ok(_lp_pool) => {
                panic!("Expected an error 'InvalidPrice', but received a successful init");
            }
        }
    }

    #[test]
    fn test_pool_initialization_doesnt_allow_fail_of_too_small_target() {
        // too small target
        match LpPool::init(Price(1__500_000), Percentage(1_000), Percentage(90_000), TokenAmount(0)) {
            Err(Errors::InvalidLiquidityTarget) => {
                assert!(true, "Error occurred as expected: InvalidLiquidityTarget");
            }
            Err(e) => {
                panic!("Unexpected error: {:?}", e);
            }
            Ok(_lp_pool) => {
                panic!("Expected an error 'InvalidLiquidityTarget', but received a successful init");
            }
        }
    }

    #[test]
    fn test_doesnt_allow_add_liquidity_of_zero() {
        let mut pool = LpPool::init(Price(1__500_000), Percentage(1_000), Percentage(90_000), TokenAmount(90__000_000)).unwrap();

        match pool.add_liquidity(TokenAmount(0)){
            Err(Errors::InvalidDeposit) => {
                assert!(true, "Error occurred as expected: InvalidDeposit");
            }
            Err(e) => {
                panic!("Unexpected error: {:?}", e);
            }
            Ok(_lp_pool) => {
                panic!("Expected an error 'InvalidDeposit'");
            }
        }

    }

    #[test]
    fn test_not_allow_swap_zero() {
        let mut pool = LpPool::init(Price(1__500_000), Percentage(1_000), Percentage(90_000), TokenAmount(90__000_000)).unwrap();

        match pool.swap(StakedTokenAmount(0)){
            Err(Errors::InvalidSwapAmount) => {
                assert!(true, "Error occurred as expected: InvalidSwapAmount");
            }
            Err(e) => {
                panic!("Unexpected error: {:?}", e);
            }
            Ok(_lp_pool) => {
                panic!("Expected an error 'InvalidSwapAmount'");
            }
        }

    }
    #[test]
    fn test_swap_not_enough_empty() {
        let mut pool = LpPool::init(Price(1__500_000), Percentage(1_000), Percentage(90_000), TokenAmount(90__000_000)).unwrap();

        match pool.swap(StakedTokenAmount(1)){
            Err(Errors::ToBigSwap(0)) => {
                assert!(true, "Error occurred as expected: ToBigSwap");
            }
            Err(e) => {
                panic!("Unexpected error: {:?}", e);
            }
            Ok(_lp_pool) => {
                panic!("Expected an error 'ToBigSwap'");
            }
        }

    }


}



fn main() {
    let mut lp = Result::unwrap(LpPool::init(
        Price(1__500_000),
        Percentage(1_000),
        Percentage(90_000),
        TokenAmount(90__000_000),
    ));
    let a = LpPool::add_liquidity(&mut lp, TokenAmount(100 * 1_000_000))
        .unwrap()
        .0;

    println!("Hello, {}", a.clone());
    println!(
        "Hello, {}",
        LpPool::swap(&mut lp, StakedTokenAmount(6 * 1_000_000))
            .unwrap()
            .0
    );
    let b = LpPool::add_liquidity(&mut lp, TokenAmount(10 * 1_000_000))
        .unwrap()
        .0;
    println!("Hello, {}", b.clone());
    println!(
        "Hello, {}",
        LpPool::swap(&mut lp, StakedTokenAmount(30 * 1_000_000))
            .unwrap()
            .0
    );
    let c = a + b;
    let (token_amount, staked_token_amount) =
        LpPool::remove_liquidity(&mut lp, LpTokenAmount(c)).unwrap();
    println!("Token Amount: {:?}", token_amount);
    println!("Staked Token Amount: {:?}", staked_token_amount);
}
