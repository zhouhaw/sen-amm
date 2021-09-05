use crate::instruction::AppInstruction;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

pub mod add_liquidity;
pub mod initialize_pool;

pub struct Processor {}

impl Processor {
  pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
  ) -> ProgramResult {
    let instruction = AppInstruction::unpack(instruction_data)?;
    match instruction {
      AppInstruction::InitializePool { delta_a, delta_b } => {
        msg!("Calling SayHello function");
        initialize_pool::exec(delta_a, delta_b, program_id, accounts)?;
        Ok(())
      }
      AppInstruction::AddLiquidity { delta_a, delta_b } => {
        msg!("Calling AddLiquidity function");
        add_liquidity::exec(delta_a, delta_b, program_id, accounts)
      }
    }
  }
}
