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
use spl_token::state::Mint;
use std::result::Result;

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

  let mut pool_data = ExoticPool::unpack(&pool_acc.data.borrow())?;
  let seed: &[&[&[u8]]] = &[&[&util::safe_seed(pool_acc, treasurer, program_id)?[..]]];
  if pool_data.is_frozen() {
    return Err(AppError::FrozenPool.into());
  }
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
  let (a_star, b_star, lpt, reserve_a, reserve_b, _) = pool_data
    .deposit(delta_a, delta_b, mint_lpt_data.supply)
    .ok_or(AppError::Overflow)?;
  // Deposit token A
  XSPLT::transfer(a_star, src_a_acc, treasury_a_acc, owner, splt_program, &[])?;
  pool_data.reserve_a = reserve_a;
  // Deposit token B
  XSPLT::transfer(b_star, src_b_acc, treasury_b_acc, owner, splt_program, &[])?;
  pool_data.reserve_b = reserve_b;
  // Update pool
  ExoticPool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;
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
