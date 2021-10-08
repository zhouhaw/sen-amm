use crate::error::AppError;
use crate::processor::{add_liquidity, swap};
use crate::schema::{pool::Pool, pool_trait::Exchange};
use num_traits::ToPrimitive;
use solana_program::{
  account_info::{next_account_info, AccountInfo},
  program_error::ProgramError,
  program_pack::Pack,
  pubkey::Pubkey,
};
use std::result::Result;

pub fn rake(
  amount: u64,
  bid_mint: &Pubkey,
  ask_mint: &Pubkey,
  pool_acc: &AccountInfo,
) -> Option<u64> {
  let pool_data = Pool::unpack(&pool_acc.data.borrow()).ok()?;
  let mut delta = amount;
  let mut bid_amount = amount.checked_div(2)?;
  loop {
    // Simulate a swap
    let (temp_ask_amount, new_bid_reserve, temp_new_ask_reserve) =
      pool_data.curve(bid_amount, bid_mint, ask_mint)?;
    let (ask_amount, fee, _) = pool_data.fee(temp_ask_amount)?;
    let new_ask_reserve = temp_new_ask_reserve.checked_add(fee)?;
    // Compute updated step
    let remainer = amount.checked_sub(bid_amount)?;
    let expected_remainer = ask_amount
      .to_u128()?
      .checked_mul(new_bid_reserve.to_u128()?)?
      .checked_div(new_ask_reserve.to_u128()?)?
      .to_u64()?;
    let next_delta = if remainer > expected_remainer {
      remainer.checked_sub(expected_remainer)?.checked_div(2)?
    } else {
      expected_remainer.checked_sub(remainer)?.checked_div(2)?
    };
    // Stop condition
    if delta > next_delta {
      delta = next_delta;
    } else {
      break;
    }
    // Converge bid amount
    bid_amount = if remainer > expected_remainer {
      bid_amount.checked_add(delta)?
    } else {
      bid_amount.checked_sub(delta)?
    };
  }
  Some(bid_amount)
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

  let taxman_acc = next_account_info(accounts_iter)?;
  let treasury_taxman_a_acc = next_account_info(accounts_iter)?;
  let treasury_taxman_b_acc = next_account_info(accounts_iter)?;

  let treasurer = next_account_info(accounts_iter)?;
  let system_program = next_account_info(accounts_iter)?;
  let splt_program = next_account_info(accounts_iter)?;
  let sysvar_rent_acc = next_account_info(accounts_iter)?;
  let splata_program = next_account_info(accounts_iter)?;

  // Pre build deposit accounts
  let deposit_accounts: [AccountInfo; 15] = [
    owner.clone(),
    pool_acc.clone(),
    lpt_acc.clone(),
    mint_lpt_acc.clone(),
    src_a_acc.clone(),
    mint_a_acc.clone(),
    treasury_a_acc.clone(),
    src_b_acc.clone(),
    mint_b_acc.clone(),
    treasury_b_acc.clone(),
    treasurer.clone(),
    system_program.clone(),
    splt_program.clone(),
    sysvar_rent_acc.clone(),
    splata_program.clone(),
  ];
  // Deposit first
  let (unraked_lpt, a_remainer, b_remainer) =
    add_liquidity::exec(delta_a, delta_b, program_id, &deposit_accounts)?;
  // Handle the remainer of A
  if a_remainer > 0 {
    let bid_amount =
      rake(a_remainer, mint_a_acc.key, mint_b_acc.key, pool_acc).ok_or(AppError::Overflow)?;
    let a = a_remainer
      .checked_sub(bid_amount)
      .ok_or(AppError::Overflow)?;
    let swap_accounts: [AccountInfo; 15] = [
      owner.clone(),
      pool_acc.clone(),
      src_a_acc.clone(),
      mint_a_acc.clone(),
      treasury_a_acc.clone(),
      src_b_acc.clone(),
      mint_b_acc.clone(),
      treasury_b_acc.clone(),
      taxman_acc.clone(),
      treasury_taxman_b_acc.clone(),
      treasurer.clone(),
      system_program.clone(),
      splt_program.clone(),
      sysvar_rent_acc.clone(),
      splata_program.clone(),
    ];
    let b = swap::exec(bid_amount, 0, program_id, &swap_accounts)?;
    let (raked_lpt, _, _) = add_liquidity::exec(a, b, program_id, &deposit_accounts)?;
    return Ok(
      unraked_lpt
        .checked_add(raked_lpt)
        .ok_or(AppError::Overflow)?,
    );
  }
  // Handle the remainer of B
  if b_remainer > 0 {
    let bid_amount =
      rake(b_remainer, mint_b_acc.key, mint_a_acc.key, pool_acc).ok_or(AppError::Overflow)?;
    let b = b_remainer
      .checked_sub(bid_amount)
      .ok_or(AppError::Overflow)?;
    let swap_accounts: [AccountInfo; 15] = [
      owner.clone(),
      pool_acc.clone(),
      src_b_acc.clone(),
      mint_b_acc.clone(),
      treasury_b_acc.clone(),
      src_a_acc.clone(),
      mint_a_acc.clone(),
      treasury_a_acc.clone(),
      taxman_acc.clone(),
      treasury_taxman_a_acc.clone(),
      treasurer.clone(),
      system_program.clone(),
      splt_program.clone(),
      sysvar_rent_acc.clone(),
      splata_program.clone(),
    ];
    let a = swap::exec(bid_amount, 0, program_id, &swap_accounts)?;
    let (raked_lpt, _, _) = add_liquidity::exec(a, b, program_id, &deposit_accounts)?;
    return Ok(
      unraked_lpt
        .checked_add(raked_lpt)
        .ok_or(AppError::Overflow)?,
    );
  }

  Ok(unraked_lpt)
}
