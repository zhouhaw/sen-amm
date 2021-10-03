use crate::error::AppError;
use crate::helper::util;
use crate::interfaces::xsplt::XSPLT;
use crate::schema::{
  exotic_pool::ExoticPool,
  pool::{Admin, Liquidity},
};
use solana_program::{
  account_info::{next_account_info, AccountInfo},
  program_error::ProgramError,
  program_pack::Pack,
  pubkey::Pubkey,
};
use std::result::Result;

pub fn exec(
  amount: u64,
  limit: u64,
  program_id: &Pubkey,
  accounts: &[AccountInfo],
) -> Result<u64, ProgramError> {
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

  let mut pool_data = ExoticPool::unpack(&pool_acc.data.borrow())?;
  let seed: &[&[&[u8]]] = &[&[&util::safe_seed(pool_acc, treasurer, program_id)?[..]]];
  if pool_data.is_frozen() {
    return Err(AppError::FrozenPool.into());
  }
  if *mint_bid_acc.key == *mint_ask_acc.key {
    return Err(AppError::SameMint.into());
  }
  if amount == 0 {
    return Err(AppError::ZeroValue.into());
  }

  let (bid_code, _) = pool_data
    .get_reserve(mint_bid_acc.key)
    .ok_or(AppError::UnmatchedPool)?;
  let (ask_code, _) = pool_data
    .get_reserve(mint_ask_acc.key)
    .ok_or(AppError::UnmatchedPool)?;

  let bid_amount = amount;
  let (pre_ask_amount, new_reserve_bid, pre_new_reserve_ask) = pool_data
    .curve(bid_amount, mint_bid_acc.key, mint_ask_acc.key)
    .ok_or(AppError::Overflow)?;
  let (ask_amount, fee, tax) = pool_data.fee(pre_ask_amount).ok_or(AppError::Overflow)?;
  let new_reserve_ask = pre_new_reserve_ask
    .checked_add(fee)
    .ok_or(AppError::Overflow)?;

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
  util::checked_transfer_splt(
    tax,
    owner,
    treasury_ask_acc,
    treasurer,
    treasury_taxman_acc,
    taxman_acc,
    mint_ask_acc,
    system_program,
    splt_program,
    sysvar_rent_acc,
    splata_program,
    seed,
  )?;
  // Execute ask (Initialize ask account if not exsting)
  util::checked_transfer_splt(
    ask_amount,
    owner,
    treasury_ask_acc,
    treasurer,
    dst_ask_acc,
    owner,
    mint_ask_acc,
    system_program,
    splt_program,
    sysvar_rent_acc,
    splata_program,
    seed,
  )?;
  match ask_code {
    0 => pool_data.reserve_a = new_reserve_ask,
    1 => pool_data.reserve_b = new_reserve_ask,
    _ => return Err(AppError::UnmatchedPool.into()),
  }
  // Update pool
  ExoticPool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;
  Ok(ask_amount)
}
