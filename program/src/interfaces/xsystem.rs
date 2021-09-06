use solana_program::{
  account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke,
  program_error::ProgramError, pubkey::Pubkey, rent::Rent, system_instruction, sysvar::Sysvar,
};

pub struct XSystem {}

impl XSystem {
  ///
  /// Check account
  ///
  pub fn check_account(target_acc: &AccountInfo) -> Result<bool, ProgramError> {
    Ok((&target_acc.data.borrow()).len() != 0)
  }
  ///
  /// Rent account
  ///
  pub fn rent_account<'a>(
    space: usize,
    target_acc: &AccountInfo<'a>,
    payer_acc: &AccountInfo<'a>,
    owner_program_id: &Pubkey,
    sysvar_rent_acc: &AccountInfo<'a>,
    system_acc: &AccountInfo<'a>,
  ) -> ProgramResult {
    // Fund the associated token account with the minimum balance to be rent exempt
    msg!("0");
    let rent = &Rent::from_account_info(sysvar_rent_acc)?;
    let required_lamports = rent
      .minimum_balance(space)
      .max(1)
      .saturating_sub(target_acc.lamports());

    msg!("1");
    if required_lamports > 0 {
      invoke(
        &system_instruction::transfer(payer_acc.key, target_acc.key, required_lamports),
        &[payer_acc.clone(), target_acc.clone(), system_acc.clone()],
      )?;
    }

    msg!("2");
    invoke(
      &system_instruction::allocate(target_acc.key, space as u64),
      &[target_acc.clone(), target_acc.clone(), system_acc.clone()],
    )?;

    msg!("3");
    invoke(
      &system_instruction::assign(target_acc.key, owner_program_id),
      &[target_acc.clone(), target_acc.clone(), system_acc.clone()],
    )?;
    Ok(())
  }
}
