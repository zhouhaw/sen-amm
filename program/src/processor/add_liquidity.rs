use crate::error::AppError;
use crate::helper::{math::U128Roots, util};
use crate::interfaces::xsplt::XSPLT;
use crate::schema::pool::Pool;
use num_traits::ToPrimitive;
use solana_program::{
  account_info::{next_account_info, AccountInfo},
  program_error::ProgramError,
  program_pack::Pack,
  pubkey::Pubkey,
};
use spl_token::state::Mint;
use std::result::Result;

///
/// Just take the correct ratio of tokens
/// Return the rest
///
pub fn rake(a: u64, b: u64, reserve_a: u64, reserve_b: u64) -> Option<(u64, u64)> {
  if a == 0 || b == 0 || reserve_a == 0 || reserve_b == 0 {
    return None;
  }
  let l = a.to_u128()?.checked_mul(reserve_b.to_u128()?)?;
  let r = b.to_u128()?.checked_mul(reserve_a.to_u128()?)?;
  // [a] > [b]
  if l > r {
    let a_star = r.checked_div(reserve_b.to_u128()?)?.to_u64()?;
    return Some((a_star, b));
  }
  // [a] < [b]
  else if l < r {
    let b_star = l.checked_div(reserve_a.to_u128()?)?.to_u64()?;
    return Some((a, b_star));
  }
  // [a] = [b]
  else {
    return Some((a, b));
  }
}

pub fn deposit(
  delta_a: u64,
  delta_b: u64,
  reserve_a: u64,
  reserve_b: u64,
  liquidity: u64,
) -> Option<(u64, u64, u64, u64, u64)> {
  // The pool hasn't initialized the reserves
  if reserve_a == 0 && reserve_b == 0 {
    let lpt = delta_a
      .to_u128()?
      .checked_mul(delta_b.to_u128()?)?
      .sqrt()
      .to_u64()?;
    return Some((delta_a, delta_b, delta_a, delta_b, lpt));
  }
  // The pool of non-empty reserves
  else {
    let (a, b) = rake(delta_a, delta_b, reserve_a, reserve_b)?;
    let lpt = a
      .to_u128()?
      .checked_mul(liquidity.to_u128()?)?
      .checked_div(reserve_a.to_u128()?)?
      .to_u64()?;
    let new_reserve_a = a.checked_add(reserve_a)?;
    let new_reserve_b = b.checked_add(reserve_b)?;
    return Some((a, b, new_reserve_a, new_reserve_b, lpt));
  }
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
  let mint_lpt_data = Mint::unpack(&mint_lpt_acc.data.borrow())?;
  let (a_star, b_star, reserve_a, reserve_b, lpt) = deposit(
    delta_a,
    delta_b,
    pool_data.reserve_a,
    pool_data.reserve_b,
    mint_lpt_data.supply,
  )
  .ok_or(AppError::Overflow)?;
  // Deposit token A
  XSPLT::transfer(a_star, src_a_acc, treasury_a_acc, owner, splt_program, &[])?;
  pool_data.reserve_a = reserve_a;
  // Deposit token B
  XSPLT::transfer(b_star, src_b_acc, treasury_b_acc, owner, splt_program, &[])?;
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
