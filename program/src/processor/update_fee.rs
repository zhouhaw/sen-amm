use crate::helper::util;
use crate::schema::{pool::Pool, pool_trait::Operation};
use solana_program::{
  account_info::{next_account_info, AccountInfo},
  program_error::ProgramError,
  program_pack::Pack,
  pubkey::Pubkey,
};
use std::result::Result;

pub fn exec(
  fee_ratio: u64,
  tax_ratio: u64,
  program_id: &Pubkey,
  accounts: &[AccountInfo],
) -> Result<(), ProgramError> {
  let accounts_iter = &mut accounts.iter();
  let owner = next_account_info(accounts_iter)?;
  let pool_acc = next_account_info(accounts_iter)?;

  util::is_program(program_id, &[pool_acc])?;
  util::is_signer(&[owner])?;

  // Update pool data
  let mut pool_data = Pool::unpack(&pool_acc.data.borrow())?;
  pool_data.is_owner(*owner.key)?;
  pool_data.fee_ratio = fee_ratio;
  pool_data.tax_ratio = tax_ratio;
  Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;

  Ok(())
}
