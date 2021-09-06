use crate::error::AppError;
use crate::helper::{oracle, pubutil::Boolean, util};
use crate::interfaces::{xsplata::XSPLATA, xsplt::XSPLT, xsystem::XSystem};
use crate::schema::pool::{Pool, PoolState};
use solana_program::{
  account_info::{next_account_info, AccountInfo},
  program_error::ProgramError,
  program_pack::{IsInitialized, Pack},
  pubkey::Pubkey,
};
use spl_token::state::{Account, Mint};
use std::result::Result;

pub fn exec(
  delta_a: u64,
  delta_b: u64,
  program_id: &Pubkey,
  accounts: &[AccountInfo],
) -> Result<u64, ProgramError> {
  let accounts_iter = &mut accounts.iter();
  let payer = next_account_info(accounts_iter)?;
  let owner = next_account_info(accounts_iter)?;
  let pool_acc = next_account_info(accounts_iter)?;
  let lpt_acc = next_account_info(accounts_iter)?;
  let mint_lpt_acc = next_account_info(accounts_iter)?;
  let taxman_acc = next_account_info(accounts_iter)?;
  let proof_acc = next_account_info(accounts_iter)?; // program_id xor treasurer xor pool_id

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

  util::is_signer(&[payer, pool_acc, mint_lpt_acc])?;

  let seed: &[&[&[u8]]] = &[&[&util::safe_seed(pool_acc, treasurer, program_id)?[..]]];
  if *proof_acc.key != program_id.xor(&(pool_acc.key.xor(treasurer.key))) {
    return Err(AppError::InvalidLpProof.into());
  }
  if *mint_a_acc.key == *mint_b_acc.key {
    return Err(AppError::SameMint.into());
  }
  if delta_a == 0 || delta_b == 0 {
    return Err(AppError::ZeroValue.into());
  }

  // Initialize treasury A
  if !XSystem::check_account(treasury_a_acc)? {
    XSystem::rent_account(
      Account::LEN,
      treasury_a_acc,
      payer,
      splt_program.key,
      sysvar_rent_acc,
      system_program,
    )?;
  }
  XSPLATA::initialize_account(
    payer,
    treasury_a_acc,
    treasurer,
    mint_a_acc,
    system_program,
    splt_program,
    sysvar_rent_acc,
    splata_program,
    &[],
  )?;
  // Deposit token A
  XSPLT::transfer(delta_a, src_a_acc, treasury_a_acc, payer, splt_program, &[])?;

  // Initialize treasury B
  if !XSystem::check_account(treasury_b_acc)? {
    XSystem::rent_account(
      Account::LEN,
      treasury_b_acc,
      payer,
      splt_program.key,
      sysvar_rent_acc,
      system_program,
    )?;
  }
  XSPLATA::initialize_account(
    payer,
    treasury_b_acc,
    treasurer,
    mint_b_acc,
    system_program,
    splt_program,
    sysvar_rent_acc,
    splata_program,
    &[],
  )?;
  // Deposit token B
  XSPLT::transfer(delta_b, src_b_acc, treasury_b_acc, payer, splt_program, &[])?;

  // Initialize mint LP
  if !XSystem::check_account(mint_lpt_acc)? {
    XSystem::rent_account(
      Mint::LEN,
      mint_lpt_acc,
      payer,
      splt_program.key,
      sysvar_rent_acc,
      system_program,
    )?;
  }
  XSPLT::initialize_mint(
    9,
    mint_lpt_acc,
    treasurer,
    proof_acc,
    sysvar_rent_acc,
    splt_program,
    seed,
  )?;

  // Initialize lpt account
  if !XSystem::check_account(lpt_acc)? {
    XSystem::rent_account(
      Account::LEN,
      lpt_acc,
      payer,
      splt_program.key,
      sysvar_rent_acc,
      system_program,
    )?;
  }
  XSPLATA::initialize_account(
    payer,
    lpt_acc,
    payer,
    mint_lpt_acc,
    system_program,
    splt_program,
    sysvar_rent_acc,
    splata_program,
    &[],
  )?;
  // Mint LPT
  let (lpt, _, _, _) = oracle::deposit(delta_a, delta_b, 0, 0).ok_or(AppError::Overflow)?;
  XSPLT::mint_to(lpt, mint_lpt_acc, lpt_acc, treasurer, splt_program, seed)?;

  // Initialize pool account
  if !XSystem::check_account(pool_acc)? {
    XSystem::rent_account(
      Pool::LEN,
      pool_acc,
      payer,
      program_id,
      sysvar_rent_acc,
      system_program,
    )?;
  }
  util::is_program(program_id, &[pool_acc])?;
  let mut pool_data = Pool::unpack_unchecked(&pool_acc.data.borrow())?;
  if pool_data.is_initialized() {
    return Err(AppError::AlreadyInitialized.into());
  }
  // Update pool data
  pool_data.owner = *owner.key;
  pool_data.state = PoolState::Initialized;
  pool_data.mint_lpt = *mint_lpt_acc.key;
  pool_data.taxman = *taxman_acc.key;
  pool_data.mint_a = *mint_a_acc.key;
  pool_data.treasury_a = *treasury_a_acc.key;
  pool_data.reserve_a = delta_a;
  pool_data.mint_b = *mint_b_acc.key;
  pool_data.treasury_b = *treasury_b_acc.key;
  pool_data.reserve_b = delta_b;
  Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;

  Ok(lpt)
}
