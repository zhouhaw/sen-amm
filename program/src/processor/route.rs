use solana_program::{
    account_info::{next_account_info, AccountInfo},
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::processor::swap;
use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

/// COMMON_ACCOUNT_LEN that means the number of accounts shared for the execution of swaps command
/// such as: owner, system_program, splt_program, sysvar_rent_acc, splata_program
const COMMON_ACCOUNT_SWAP_LEN: u8 = 5;

pub fn exec(
    amount: u64,
    limit: u64,
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> Result<u64, ProgramError> {
    let accounts_len = accounts.len();
    let mut ask_amount = amount;

    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let splt_program = next_account_info(accounts_iter)?;
    let sysvar_rent_acc = next_account_info(accounts_iter)?;
    let splata_program = next_account_info(accounts_iter)?;

    /// In addition to the shared accounts above, we need 10 more detailed accounts below
    loop {
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
        ask_amount = swap::exec(amount, limit, program_id, &swap_accounts)?;
    }

    Ok(ask_amount);
}
