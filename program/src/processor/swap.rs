use crate::error::AppError;
use crate::helper::{oracle, util};
use crate::interfaces::{xsplata::XSPLATA, xsplt::XSPLT, xsystem::XSystem};
use crate::schema::pool::Pool;
use solana_program::{
  account_info::{next_account_info, AccountInfo},
  entrypoint::ProgramResult,
  program_pack::{IsInitialized, Pack},
  pubkey::Pubkey,
};
use spl_token::state::Account;

pub fn exec(
  amount: u64,
  limit: u64,
  program_id: &Pubkey,
  accounts: &[AccountInfo],
) -> ProgramResult {
  let accounts_iter = &mut accounts.iter();
  let owner = next_account_info(accounts_iter)?;
  let pool_acc = next_account_info(accounts_iter)?;

  let src_bid_acc = next_account_info(accounts_iter)?;
  let mint_bid_acc = next_account_info(accounts_iter)?;
  let treasury_bid_acc = next_account_info(accounts_iter)?;

  let dst_ask_acc = next_account_info(accounts_iter)?;
  let mint_ask_acc = next_account_info(accounts_iter)?;
  let treasury_ask_acc = next_account_info(accounts_iter)?;

  let taxman_acc = next_account_info(accounts_iter)?;
  let treasury_taxman_acc = next_account_info(accounts_iter)?;

  let treasurer = next_account_info(accounts_iter)?;
  let system_program = next_account_info(accounts_iter)?;
  let splt_program = next_account_info(accounts_iter)?;
  let sysvar_rent_acc = next_account_info(accounts_iter)?;
  let splata_program = next_account_info(accounts_iter)?;

  util::is_program(program_id, &[pool_acc])?;
  util::is_signer(&[owner])?;

  let mut pool_data = Pool::unpack(&pool_acc.data.borrow())?;
  let seed: &[&[&[u8]]] = &[&[&util::safe_seed(pool_acc, treasurer, program_id)?[..]]];
  if *mint_bid_acc.key == *mint_ask_acc.key {
    return Err(AppError::SameMint.into());
  }
  if amount == 0 {
    return Err(AppError::ZeroValue.into());
  }

  let (bid_code, reserve_bid) = pool_data
    .get_reserve(mint_bid_acc.key)
    .ok_or(AppError::UnmatchedPool)?;
  let (ask_code, reserve_ask) = pool_data
    .get_reserve(mint_ask_acc.key)
    .ok_or(AppError::UnmatchedPool)?;

  let bid_amount = amount;
  let (ask_amount, tax, new_reserve_bid, new_reserve_ask) =
    oracle::swap(bid_amount, reserve_bid, reserve_ask).ok_or(AppError::Overflow)?;

  if ask_amount < limit {
    return Err(AppError::ExceedLimit.into());
  }

  // Execute bid
  XSPLT::transfer(
    bid_amount,
    src_bid_acc,
    treasury_bid_acc,
    owner,
    splt_program,
    &[],
  )?;
  match bid_code {
    0 => pool_data.reserve_a = new_reserve_bid,
    1 => pool_data.reserve_b = new_reserve_bid,
    _ => return Err(AppError::UnmatchedPool.into()),
  }
  // Pay tax (Initialize ask account if not exsting)
  if !XSystem::check_account(treasury_taxman_acc)? {
    XSystem::rent_account(
      Account::LEN,
      treasury_taxman_acc,
      owner,
      splt_program.key,
      sysvar_rent_acc,
      system_program,
    )?;
  }
  let acc_data = Account::unpack_unchecked(&treasury_taxman_acc.data.borrow())?;
  if !acc_data.is_initialized() {
    XSPLATA::initialize_account(
      owner,
      treasury_taxman_acc,
      taxman_acc,
      mint_ask_acc,
      system_program,
      splt_program,
      sysvar_rent_acc,
      splata_program,
      &[],
    )?;
  }
  XSPLT::transfer(
    tax,
    treasury_ask_acc,
    treasury_taxman_acc,
    treasurer,
    splt_program,
    seed,
  )?;
  // Execute ask (Initialize ask account if not exsting)
  if !XSystem::check_account(dst_ask_acc)? {
    XSystem::rent_account(
      Account::LEN,
      dst_ask_acc,
      owner,
      splt_program.key,
      sysvar_rent_acc,
      system_program,
    )?;
  }
  let acc_data = Account::unpack_unchecked(&dst_ask_acc.data.borrow())?;
  if !acc_data.is_initialized() {
    XSPLATA::initialize_account(
      owner,
      dst_ask_acc,
      owner,
      mint_ask_acc,
      system_program,
      splt_program,
      sysvar_rent_acc,
      splata_program,
      &[],
    )?;
  }
  XSPLT::transfer(
    ask_amount,
    treasury_ask_acc,
    dst_ask_acc,
    treasurer,
    splt_program,
    seed,
  )?;
  match ask_code {
    0 => pool_data.reserve_a = new_reserve_ask,
    1 => pool_data.reserve_b = new_reserve_ask,
    _ => return Err(AppError::UnmatchedPool.into()),
  }
  // Update pool
  Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;
  Ok(())
}
