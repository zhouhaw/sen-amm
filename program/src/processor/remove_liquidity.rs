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

pub fn withdraw(
  delta_liquidity: u64,
  reserve_a: u64,
  reserve_b: u64,
) -> Option<(u64, u64, u64, u64)> {
  let liquidity = (reserve_a as u128).checked_mul(reserve_b as u128)?.sqrt() as u64;
  let delta_a = (reserve_a as u128)
    .checked_mul(delta_liquidity as u128)?
    .checked_div(liquidity as u128)? as u64;
  let delta_b = (reserve_b as u128)
    .checked_mul(delta_liquidity as u128)?
    .checked_div(liquidity as u128)? as u64;
  let new_reserve_a = reserve_a.checked_sub(delta_a)?;
  let new_reserve_b = reserve_b.checked_sub(delta_b)?;
  Some((delta_a, delta_b, new_reserve_a, new_reserve_b))
}

pub fn exec(
  lpt: u64,
  program_id: &Pubkey,
  accounts: &[AccountInfo],
) -> Result<(u64, u64), ProgramError> {
  let accounts_iter = &mut accounts.iter();
  let owner = next_account_info(accounts_iter)?;
  let pool_acc = next_account_info(accounts_iter)?;
  let lpt_acc = next_account_info(accounts_iter)?;
  let mint_lpt_acc = next_account_info(accounts_iter)?;

  let dst_a_acc = next_account_info(accounts_iter)?;
  let mint_a_acc = next_account_info(accounts_iter)?;
  let treasury_a_acc = next_account_info(accounts_iter)?;

  let dst_b_acc = next_account_info(accounts_iter)?;
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
  if lpt == 0 {
    return Err(AppError::ZeroValue.into());
  }

  // Burn lpt
  let (delta_a, delta_b, reserve_a, reserve_b) =
    withdraw(lpt, pool_data.reserve_a, pool_data.reserve_b).ok_or(AppError::Overflow)?;
  XSPLT::burn(lpt, lpt_acc, mint_lpt_acc, owner, splt_program, seed)?;
  // Update pool
  pool_data.reserve_a = reserve_a;
  pool_data.reserve_b = reserve_b;
  Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;
  // Withdraw A
  util::checked_transfer_splt(
    delta_a,
    owner,
    treasury_a_acc,
    treasurer,
    dst_a_acc,
    owner,
    mint_a_acc,
    system_program,
    splt_program,
    sysvar_rent_acc,
    splata_program,
    seed,
  )?;
  // Withdraw B
  util::checked_transfer_splt(
    delta_b,
    owner,
    treasury_b_acc,
    treasurer,
    dst_b_acc,
    owner,
    mint_b_acc,
    system_program,
    splt_program,
    sysvar_rent_acc,
    splata_program,
    seed,
  )?;

  Ok((delta_a, delta_b))
}
