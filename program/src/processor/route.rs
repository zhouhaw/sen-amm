use crate::error::AppError;
use crate::processor::swap;
use solana_program::{
  account_info::{next_account_info, AccountInfo},
  program_error::ProgramError,
  pubkey::Pubkey,
};

pub fn exec(
  amount: u64,
  limit: u64,
  program_id: &Pubkey,
  accounts: &[AccountInfo],
) -> Result<u64, ProgramError> {
  let accounts_iter = &mut accounts.iter();
  let owner = next_account_info(accounts_iter)?;
  let system_program = next_account_info(accounts_iter)?;
  let splt_program = next_account_info(accounts_iter)?;
  let sysvar_rent_acc = next_account_info(accounts_iter)?;
  let splata_program = next_account_info(accounts_iter)?;

  // In addition to the shared accounts above, we need 10 more detailed accounts below
  let mut ask_amount = amount;
  while accounts_iter.len() != 0 {
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

    let swap_accounts: [AccountInfo; 15] = [
      owner.clone(),
      pool_acc.clone(),
      src_bid_acc.clone(),
      mint_bid_acc.clone(),
      treasury_bid_acc.clone(),
      dst_ask_acc.clone(),
      mint_ask_acc.clone(),
      treasury_ask_acc.clone(),
      taxman_acc.clone(),
      treasury_taxman_acc.clone(),
      treasurer.clone(),
      system_program.clone(),
      splt_program.clone(),
      sysvar_rent_acc.clone(),
      splata_program.clone(),
    ];
    ask_amount = swap::exec(ask_amount, 0, program_id, &swap_accounts)?;
  }

  if ask_amount < limit {
    return Err(AppError::ExceedLimit.into());
  }

  Ok(ask_amount)
}
