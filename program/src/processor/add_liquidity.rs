#![allow(non_snake_case)]

use crate::error::AppError;
use crate::helper::{math::Roots, util};
use crate::interfaces::xsplt::XSPLT;
use crate::schema::pool::Pool;
use solana_program::{
  account_info::{next_account_info, AccountInfo},
  program_error::ProgramError,
  program_pack::Pack,
  pubkey::Pubkey,
};
use std::result::Result;

///
/// Assume a/b > A/B means [a] > [b]
/// We need to sell [a] and buy [b] to rebalance
///
pub fn rake(a: u64, b: u64, A: u64, B: u64) -> Option<(u64, u64, u64, u64)> {
  if (a as u128).checked_mul(B as u128)? < (b as u128).checked_mul(A as u128)? {
    return None;
  }
  if A == 0 && B == 0 {
    return Some((a, b, A, B)); // Empty pool
  }
  let aB = (a as u128).checked_mul(B as u128)?; // a*B
  let bA = (b as u128).checked_mul(A as u128)?; // b*A
  let aB_bA = aB.checked_sub(bA)?; // a*B - b*A
  let a_A = (a as u128).checked_add(A as u128)?; // a + A
  let b_B = (b as u128).checked_add(B as u128)?; // b + B
  let a_hat = aB_bA.checked_div(b_B)?.checked_div(2 as u128)? as u64; // (a*B - b*A) / [2(b + B)]
  let b_hat = aB_bA.checked_div(a_A)?.checked_div(2 as u128)? as u64; // (a*B - b*A) / [2(a + A)]
  let a_star = a.checked_sub(a_hat)?; // a_star = a - a_hat
  let b_star = b.checked_add(b_hat)?; // b_star = b + b_hat
  let A_star = A.checked_add(a_hat)?; // A_star = A + a_hat
  let B_star = B.checked_sub(b_hat)?; // B_star = B - b_hat
  Some((a_star, b_star, A_star, B_star)) // At this: a_star / b_star = A_star / B_star
}

pub fn deposit(
  delta_a: u64,
  delta_b: u64,
  reserve_a: u64,
  reserve_b: u64,
) -> Option<(u64, u64, u64, u64)> {
  let delta_liquidity = (delta_a as u128).checked_mul(delta_b as u128)?.sqrt() as u64;
  let liquidity = (reserve_a as u128).checked_mul(reserve_b as u128)?.sqrt() as u64;
  let new_liquidity = delta_liquidity.checked_add(liquidity)?;
  let new_reserve_a = delta_a.checked_add(reserve_a)?;
  let new_reserve_b = delta_b.checked_add(reserve_b)?;
  Some((delta_liquidity, new_liquidity, new_reserve_a, new_reserve_b))
}

pub fn exec(
  delta_a: u64,
  delta_b: u64,
  program_id: &Pubkey,
  accounts: &[AccountInfo],
) -> Result<u64, ProgramError> {
  let accounts_iter = &mut accounts.iter();
  let owner = next_account_info(accounts_iter)?;
  let pool_acc = next_account_info(accounts_iter)?;
  let lpt_acc = next_account_info(accounts_iter)?;
  let mint_lpt_acc = next_account_info(accounts_iter)?;

  let src_a_acc = next_account_info(accounts_iter)?;
  let mint_a_acc = next_account_info(accounts_iter)?;
  let treasury_a_acc = next_account_info(accounts_iter)?;

  let src_b_acc = next_account_info(accounts_iter)?;
  let mint_b_acc = next_account_info(accounts_iter)?;
  let treasury_b_acc = next_account_info(accounts_iter)?;

  let treasurer = next_account_info(accounts_iter)?;
  let system_program = next_account_info(accounts_iter)?;
  let splt_program = next_account_info(accounts_iter)?;
  let sysvar_rent_acc = next_account_info(accounts_iter)?;
  let splata_program = next_account_info(accounts_iter)?;

  util::is_program(program_id, &[pool_acc])?;
  util::is_signer(&[owner])?;

  let mut pool_data = Pool::unpack(&pool_acc.data.borrow())?;
  let seed: &[&[&[u8]]] = &[&[&util::safe_seed(pool_acc, treasurer, program_id)?[..]]];
  if pool_data.mint_lpt != *mint_lpt_acc.key
    || pool_data.mint_a != *mint_a_acc.key
    || pool_data.mint_b != *mint_b_acc.key
    || pool_data.treasury_a != *treasury_a_acc.key
    || pool_data.treasury_b != *treasury_b_acc.key
  {
    return Err(AppError::UnmatchedPool.into());
  }
  if delta_a == 0 && delta_b == 0 {
    return Err(AppError::ZeroValue.into());
  }

  // Balance the deposit
  let L = (delta_a as u128)
    .checked_mul(pool_data.reserve_b as u128)
    .ok_or(AppError::Overflow)?;
  let R = (delta_b as u128)
    .checked_mul(pool_data.reserve_a as u128)
    .ok_or(AppError::Overflow)?;
  let (delta_a_star, delta_b_star, reserve_a_star, reserve_b_star) = if L > R {
    let (a_star, b_star, A_star, B_star) =
      rake(delta_a, delta_b, pool_data.reserve_a, pool_data.reserve_b).ok_or(AppError::Overflow)?;
    (a_star, b_star, A_star, B_star)
  } else if L < R {
    let (b_star, a_star, B_star, A_star) =
      rake(delta_b, delta_a, pool_data.reserve_b, pool_data.reserve_a).ok_or(AppError::Overflow)?;
    (a_star, b_star, A_star, B_star)
  } else {
    let (a_star, b_star, A_star, B_star) =
      (delta_a, delta_b, pool_data.reserve_a, pool_data.reserve_b);
    (a_star, b_star, A_star, B_star)
  };
  // Accept the deposit
  let (lpt, _, reserve_a, reserve_b) =
    deposit(delta_a_star, delta_b_star, reserve_a_star, reserve_b_star)
      .ok_or(AppError::Overflow)?;
  // Deposit token A
  XSPLT::transfer(delta_a, src_a_acc, treasury_a_acc, owner, splt_program, &[])?;
  pool_data.reserve_a = reserve_a;
  // Deposit token B
  XSPLT::transfer(delta_b, src_b_acc, treasury_b_acc, owner, splt_program, &[])?;
  pool_data.reserve_b = reserve_b;
  // Update pool
  Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;
  // Initialize lpt account
  util::checked_initialize_splt_account(
    owner,
    lpt_acc,
    owner,
    mint_lpt_acc,
    system_program,
    splt_program,
    sysvar_rent_acc,
    splata_program,
  )?;
  // Mint LPT
  XSPLT::mint_to(lpt, mint_lpt_acc, lpt_acc, treasurer, splt_program, seed)?;

  Ok(lpt)
}
