use crate::error::AppError;
use solana_program::program_error::ProgramError;
use std::convert::TryInto;

#[derive(Clone, Debug, PartialEq)]
pub enum AppInstruction {
  InitializePool {
    delta_a: u64,
    delta_b: u64,
    fee_ratio: u64,
    tax_ratio: u64,
  },
  AddLiquidity {
    delta_a: u64,
    delta_b: u64,
  },
  RemoveLiquidity {
    lpt: u64,
  },
  Swap {
    amount: u64,
    limit: u64,
  },
  FreezePool,
  ThawPool,
  TransferTaxman,
  TransferOwnership,
  Route {
    amount: u64,
    limit: u64,
  },
  UpdateFee {
    fee_ratio: u64,
    tax_ratio: u64,
  },
  AddSidedLiquidity {
    delta_a: u64,
    delta_b: u64,
  },
}

impl AppInstruction {
  pub fn unpack(instruction: &[u8]) -> Result<Self, ProgramError> {
    let (&tag, rest) = instruction
      .split_first()
      .ok_or(AppError::InvalidInstruction)?;
    Ok(match tag {
      0 => {
        let delta_a = rest
          .get(..8)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        let delta_b = rest
          .get(8..16)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        let fee_ratio = rest
          .get(16..24)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        let tax_ratio = rest
          .get(24..32)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        Self::InitializePool {
          delta_a,
          delta_b,
          fee_ratio,
          tax_ratio,
        }
      }
      1 => {
        let delta_a = rest
          .get(..8)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        let delta_b = rest
          .get(8..16)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        Self::AddLiquidity { delta_a, delta_b }
      }
      2 => {
        let lpt = rest
          .get(..8)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        Self::RemoveLiquidity { lpt }
      }
      3 => {
        let amount = rest
          .get(..8)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        let limit = rest
          .get(8..16)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        Self::Swap { amount, limit }
      }
      4 => Self::FreezePool,
      5 => Self::ThawPool,
      6 => Self::TransferTaxman,
      7 => Self::TransferOwnership,
      8 => {
        let amount = rest
          .get(..8)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        let limit = rest
          .get(8..16)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        Self::Route { amount, limit }
      }
      9 => {
        let fee_ratio = rest
          .get(..8)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        let tax_ratio = rest
          .get(8..16)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        Self::UpdateFee {
          fee_ratio,
          tax_ratio,
        }
      }
      10 => {
        let delta_a = rest
          .get(..8)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        let delta_b = rest
          .get(8..16)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        Self::AddSidedLiquidity { delta_a, delta_b }
      }
      _ => return Err(AppError::InvalidInstruction.into()),
    })
  }
}
