use crate::error::AppError;
use crate::schema::pool::Pool;
use solana_program::{
  account_info::AccountInfo,
  entrypoint::ProgramResult,
  program_pack::Pack,
  pubkey::{Pubkey, PubkeyError},
};

pub fn is_program(program_id: &Pubkey, accounts: &[&AccountInfo]) -> ProgramResult {
  for acc in &mut accounts.iter() {
    if acc.owner != program_id {
      return Err(AppError::IncorrectProgramId.into());
    }
  }
  Ok(())
}

pub fn is_signer(accounts: &[&AccountInfo]) -> ProgramResult {
  for acc in &mut accounts.iter() {
    if !acc.is_signer {
      return Err(AppError::InvalidOwner.into());
    }
  }
  Ok(())
}

pub fn is_pool_owner(owner: &AccountInfo, pool_acc: &AccountInfo) -> ProgramResult {
  let pool_data = Pool::unpack(&pool_acc.data.borrow())?;
  if pool_data.owner != *owner.key {
    return Err(AppError::InvalidOwner.into());
  }
  Ok(())
}

pub fn safe_seed(
  seed_acc: &AccountInfo,
  expected_acc: &AccountInfo,
  program_id: &Pubkey,
) -> Result<[u8; 32], PubkeyError> {
  let seed: [u8; 32] = seed_acc.key.to_bytes();
  let key = Pubkey::create_program_address(&[&seed], program_id)?;
  if key != *expected_acc.key {
    return Err(PubkeyError::InvalidSeeds);
  }
  Ok(seed)
}
