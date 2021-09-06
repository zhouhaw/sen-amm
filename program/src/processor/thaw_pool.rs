use crate::helper::util;
use crate::schema::pool::{Pool, PoolState};
use solana_program::{
  account_info::{next_account_info, AccountInfo},
  program_error::ProgramError,
  program_pack::Pack,
  pubkey::Pubkey,
};
use std::result::Result;

pub fn exec(program_id: &Pubkey, accounts: &[AccountInfo]) -> Result<(), ProgramError> {
  let accounts_iter = &mut accounts.iter();
  let owner = next_account_info(accounts_iter)?;
  let pool_acc = next_account_info(accounts_iter)?;

  util::is_program(program_id, &[pool_acc])?;
  util::is_signer(&[owner])?;
  util::is_pool_owner(owner, pool_acc)?;

  let mut pool_data = Pool::unpack(&pool_acc.data.borrow())?;
  pool_data.state = PoolState::Initialized;
  Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;

  Ok(())
}
