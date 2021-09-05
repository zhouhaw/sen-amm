use crate::helper::math::Roots;

const PRECISION: u64 = 1000000000000000000; // 10^18

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

pub fn get_liquidity(
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