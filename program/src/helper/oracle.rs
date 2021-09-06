use crate::helper::math::Roots;

const PRECISION: u64 = 1000000000000000000; // 10^18
const TAX: u64 = 500000000000000; // 0.05%

pub fn check_liquidity(delta_a: u64, delta_b: u64, reserve_a: u64, reserve_b: u64) -> Option<bool> {
  if delta_a == 0 || delta_b == 0 {
    return Some(false);
  }
  // Recently initialized pool
  if reserve_a == 0 || reserve_b == 0 {
    return Some(true);
  }
  let ratio = (delta_a as u128)
    .checked_mul(PRECISION as u128)?
    .checked_div(delta_b as u128)?;
  let expected_ratio = (reserve_a as u128)
    .checked_mul(PRECISION as u128)?
    .checked_div(reserve_b as u128)?;
  Some(ratio == expected_ratio)
}

pub fn deposit(
  delta_a: u64,
  delta_b: u64,
  reserve_a: u64,
  reserve_b: u64,
) -> Option<(u64, u64, u64, u64)> {
  if !check_liquidity(delta_a, delta_b, reserve_a, reserve_b)? {
    return None;
  }
  let delta_liquidity = (delta_a as u128).checked_mul(delta_b as u128)?.sqrt() as u64;
  let liquidity = (reserve_a as u128).checked_mul(reserve_b as u128)?.sqrt() as u64;
  let new_liquidity = delta_liquidity.checked_add(liquidity)?;
  let new_reserve_a = delta_a.checked_add(reserve_a)?;
  let new_reserve_b = delta_b.checked_add(reserve_b)?;
  Some((delta_liquidity, new_liquidity, new_reserve_a, new_reserve_b))
}

pub fn withdraw(
  delta_liquidity: u64,
  reserve_a: u64,
  reserve_b: u64,
) -> Option<(u64, u64, u64, u64, u64)> {
  let liquidity = (reserve_a as u128).checked_mul(reserve_b as u128)?.sqrt() as u64;
  let delta_a = (reserve_a as u128)
    .checked_mul(delta_liquidity as u128)?
    .checked_div(liquidity as u128)? as u64;
  let delta_b = (reserve_b as u128)
    .checked_mul(delta_liquidity as u128)?
    .checked_div(liquidity as u128)? as u64;
  let new_liquidity = liquidity.checked_sub(delta_liquidity)?;
  let new_reserve_a = reserve_a.checked_sub(delta_a)?;
  let new_reserve_b = reserve_b.checked_sub(delta_b)?;
  Some((
    delta_a,
    delta_b,
    new_liquidity,
    new_reserve_a,
    new_reserve_b,
  ))
}

pub fn adaptive_fee(ask_amount: u64, alpha: u64) -> Option<(u64, u64)> {
  let numerator = PRECISION.checked_sub(alpha)?;
  let denominator = (2 as u64).checked_mul(PRECISION)?.checked_sub(alpha)?;
  let fee = ask_amount
    .checked_mul(numerator)?
    .checked_div(denominator)?;
  let amount = ask_amount.checked_sub(fee)?;
  Some((amount, fee))
}

pub fn tax(ask_amount: u64) -> Option<(u64, u64)> {
  let tax = ask_amount.checked_mul(TAX)?.checked_div(PRECISION)?;
  let amount = ask_amount.checked_sub(tax)?;
  Some((amount, tax))
}

pub fn swap(bid_amount: u64, reserve_bid: u64, reserve_ask: u64) -> Option<(u64, u64, u64, u64)> {
  let liquidity = (reserve_bid as u128)
    .checked_mul(reserve_ask as u128)?
    .sqrt();
  let new_reserve_bid = reserve_bid.checked_add(bid_amount)?;
  let temp_reserve_ask = (liquidity).checked_div(new_reserve_bid as u128)? as u64;
  let temp_ask_amount = reserve_ask.checked_sub(temp_reserve_ask)?;
  let alpha = reserve_bid
    .checked_mul(PRECISION)?
    .checked_div(new_reserve_bid)?;
  let (_, fee) = adaptive_fee(temp_ask_amount, alpha)?;
  let (_, tax) = tax(temp_ask_amount)?;
  let ask_amount = temp_ask_amount.checked_sub(fee)?.checked_sub(tax)?;
  let new_reserve_ask = temp_reserve_ask.checked_add(fee)?;
  Some((ask_amount, tax, new_reserve_bid, new_reserve_ask))
}
