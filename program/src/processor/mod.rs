use crate::instruction::AppInstruction;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

pub mod add_liquidity;
pub mod freeze_pool;
pub mod initialize_pool;
pub mod remove_liquidity;
pub mod route;
pub mod swap;
pub mod thaw_pool;
pub mod transfer_ownership;
pub mod transfer_taxman;

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
                msg!("Calling InitializePool function");
                initialize_pool::exec(delta_a, delta_b, program_id, accounts)?;
                Ok(())
            }
            AppInstruction::AddLiquidity { delta_a, delta_b } => {
                msg!("Calling AddLiquidity function");
                add_liquidity::exec(delta_a, delta_b, program_id, accounts)?;
                Ok(())
            }
            AppInstruction::RemoveLiquidity { lpt } => {
                msg!("Calling RemoveLiquidity function");
                remove_liquidity::exec(lpt, program_id, accounts)?;
                Ok(())
            }
            AppInstruction::Swap { amount, limit } => {
                msg!("Calling Swap function");
                swap::exec(amount, limit, program_id, accounts)?;
                Ok(())
            }
            AppInstruction::FreezePool {} => {
                msg!("Calling FreezePool function");
                freeze_pool::exec(program_id, accounts)?;
                Ok(())
            }
            AppInstruction::ThawPool {} => {
                msg!("Calling ThawPool function");
                thaw_pool::exec(program_id, accounts)?;
                Ok(())
            }
            AppInstruction::TransferTaxman {} => {
                msg!("Calling TransferTaxman function");
                transfer_taxman::exec(program_id, accounts)?;
                Ok(())
            }
            AppInstruction::TransferOwnership {} => {
                msg!("Calling TransferOwnership function");
                transfer_ownership::exec(program_id, accounts)?;
                Ok(())
            } // AppInstruction::Route => code
              // accounts : pools :: pool (mints)
        }
    }
}
