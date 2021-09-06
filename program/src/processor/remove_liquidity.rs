use crate::error::AppError;
use crate::helper::{oracle, util};
use crate::interfaces::{xsplata::XSPLATA, xsplt::XSPLT, xsystem::XSystem};
use crate::schema::pool::Pool;
use solana_program::{
  account_info::{next_account_info, AccountInfo},
  program_error::ProgramError,
  program_pack::{IsInitialized, Pack},
  pubkey::Pubkey,
};
use spl_token::state::Account;
use std::result::Result;

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

  // Withdraw token
  let (delta_a, delta_b, _, reserve_a, reserve_b) =
    oracle::withdraw(lpt, pool_data.reserve_a, pool_data.reserve_b).ok_or(AppError::Overflow)?;
  XSPLT::burn(lpt, lpt_acc, mint_lpt_acc, owner, splt_program, seed)?;
  // Initialize account A
  if !XSystem::check_account(dst_a_acc)? {
    XSystem::rent_account(
      Account::LEN,
      dst_a_acc,
      owner,
      splt_program.key,
      sysvar_rent_acc,
      system_program,
    )?;
  }
  let acc_data = Account::unpack_unchecked(&dst_a_acc.data.borrow())?;
  if !acc_data.is_initialized() {
    XSPLATA::initialize_account(
      owner,
      dst_a_acc,
      owner,
      mint_a_acc,
      system_program,
      splt_program,
      sysvar_rent_acc,
      splata_program,
      &[],
    )?;
  }
  XSPLT::transfer(
    delta_a,
    treasury_a_acc,
    dst_a_acc,
    treasurer,
    splt_program,
    seed,
  )?;
  pool_data.reserve_a = reserve_a;
  // Initialize account B
  if !XSystem::check_account(dst_b_acc)? {
    XSystem::rent_account(
      Account::LEN,
      dst_b_acc,
      owner,
      splt_program.key,
      sysvar_rent_acc,
      system_program,
    )?;
  }
  let acc_data = Account::unpack_unchecked(&dst_b_acc.data.borrow())?;
  if !acc_data.is_initialized() {
    XSPLATA::initialize_account(
      owner,
      dst_b_acc,
      owner,
      mint_b_acc,
      system_program,
      splt_program,
      sysvar_rent_acc,
      splata_program,
      &[],
    )?;
  }
  XSPLT::transfer(
    delta_b,
    treasury_b_acc,
    dst_b_acc,
    treasurer,
    splt_program,
    seed,
  )?;
  pool_data.reserve_b = reserve_b;
  // Update pool
  Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;

  Ok((delta_a, delta_b))
}
