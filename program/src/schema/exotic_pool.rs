use crate::error::AppError;
use crate::helper::math::{U128Roots, PRECISION};
use crate::schema::pool::{Admin, Liquidity, PoolState};
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use num_enum::TryFromPrimitive;
use num_traits::ToPrimitive;
use solana_program::{
  msg,
  program_error::ProgramError,
  program_pack::{IsInitialized, Pack, Sealed},
  pubkey::Pubkey,
};

///
/// Just take the correct ratio of tokens
/// Return the rest
///
fn rake(a: u64, b: u64, reserve_a: u64, reserve_b: u64) -> Option<(u64, u64)> {
  if a == 0 || b == 0 || reserve_a == 0 || reserve_b == 0 {
    return None;
  }
  let l = a.to_u128()?.checked_mul(reserve_b.to_u128()?)?;
  let r = b.to_u128()?.checked_mul(reserve_a.to_u128()?)?;
  // [a] > [b]
  if l > r {
    let a_star = r.checked_div(reserve_b.to_u128()?)?.to_u64()?;
    return Some((a_star, b));
  }
  // [a] < [b]
  else if l < r {
    let b_star = l.checked_div(reserve_a.to_u128()?)?.to_u64()?;
    return Some((a, b_star));
  }
  // [a] = [b]
  else {
    return Some((a, b));
  }
}

///
/// Exotic Pool struct
///
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ExoticPool {
  pub owner: Pubkey,
  pub state: PoolState,
  pub mint_lpt: Pubkey,
  pub taxman: Pubkey,

  pub mint_a: Pubkey,
  pub treasury_a: Pubkey,
  pub reserve_a: u64,

  pub mint_b: Pubkey,
  pub treasury_b: Pubkey,
  pub reserve_b: u64,

  pub fee_ratio: u64,
  pub tax_ratio: u64,
}

///
/// Admin trait
///
impl Admin for ExoticPool {
  fn is_frozen(&self) -> bool {
    self.state == PoolState::Frozen
  }
  fn is_owner(&self, expected_owner: Pubkey) -> Result<(), ProgramError> {
    if self.owner != expected_owner {
      return Err(AppError::InvalidOwner.into());
    }
    Ok(())
  }
}

///
/// Liquidity trait
///
impl Liquidity for ExoticPool {
  // Get code and reserve
  fn get_reserve(&self, mint: &Pubkey) -> Option<(u8, u64)> {
    if self.mint_a == *mint {
      return Some((0, self.reserve_a));
    }
    if self.mint_b == *mint {
      return Some((1, self.reserve_b));
    }
    None
  }
  // Curve
  fn curve(
    &self,
    bid_amount: u64,
    bid_mint: &Pubkey,
    ask_mint: &Pubkey,
  ) -> Option<(u64, u64, u64)> {
    let (_, bid_reserve) = self.get_reserve(bid_mint)?;
    let (_, ask_reserve) = self.get_reserve(ask_mint)?;
    let new_bid_reserve = bid_reserve.checked_add(bid_amount)?;
    let new_ask_reserve = bid_reserve
      .to_u128()?
      .checked_mul(ask_reserve.to_u128()?)?
      .checked_div(new_bid_reserve.to_u128()?)?
      .to_u64()?;
    let ask_amount = ask_reserve.checked_sub(new_ask_reserve)?;
    Some((ask_amount, new_bid_reserve, new_ask_reserve))
  }
  // Fee
  fn fee(&self, ask_amount: u64) -> Option<(u64, u64, u64)> {
    let fee = self
      .fee_ratio
      .to_u128()?
      .checked_mul(ask_amount.to_u128()?)?
      .checked_div(PRECISION.to_u128()?)?
      .to_u64()?;
    let temp_amount = ask_amount.checked_sub(fee)?;
    let tax = self
      .tax_ratio
      .to_u128()?
      .checked_mul(temp_amount.to_u128()?)?
      .checked_div(PRECISION.to_u128()?)?
      .to_u64()?;
    let amount = temp_amount.checked_sub(tax)?;
    Some((amount, fee, tax))
  }
  // Add liquidity
  fn deposit(
    &self,
    delta_a: u64,
    delta_b: u64,
    liquidity: u64,
  ) -> Option<(u64, u64, u64, u64, u64, u64)> {
    // The pool hasn't initialized the reserves
    if self.reserve_a == 0 && self.reserve_b == 0 {
      let lpt = delta_a
        .to_u128()?
        .checked_mul(delta_b.to_u128()?)?
        .sqrt()
        .to_u64()?;
      return Some((delta_a, delta_b, lpt, delta_a, delta_b, lpt));
    }
    // The pool of non-empty reserves
    let (a, b) = rake(delta_a, delta_b, self.reserve_a, self.reserve_b)?;
    let new_reserve_a = a.checked_add(self.reserve_a)?;
    let new_reserve_b = b.checked_add(self.reserve_b)?;
    let lpt = a
      .to_u128()?
      .checked_mul(liquidity.to_u128()?)?
      .checked_div(self.reserve_a.to_u128()?)?
      .to_u64()?;
    let new_liquidity = liquidity.checked_add(lpt)?;
    Some((a, b, lpt, new_reserve_a, new_reserve_b, new_liquidity))
  }
  // Remove liquidity
  fn withdraw(&self, lpt: u64, liquidity: u64) -> Option<(u64, u64, u64, u64, u64, u64)> {
    let new_liquidity = liquidity.checked_sub(lpt)?;
    let new_reserve_a = new_liquidity
      .to_u128()?
      .checked_mul(self.reserve_a.to_u128()?)?
      .checked_div(liquidity.to_u128()?)?
      .to_u64()?;
    let new_reserve_b = new_liquidity
      .to_u128()?
      .checked_mul(self.reserve_b.to_u128()?)?
      .checked_div(liquidity.to_u128()?)?
      .to_u64()?;
    let delta_a = self.reserve_a.checked_sub(new_reserve_a)?;
    let delta_b = self.reserve_b.checked_sub(new_reserve_b)?;
    Some((
      delta_a,
      delta_b,
      lpt,
      new_reserve_a,
      new_reserve_b,
      new_liquidity,
    ))
  }
}

///
/// Sealed trait
///
impl Sealed for ExoticPool {}

///
/// IsInitialized trait
///
impl IsInitialized for ExoticPool {
  fn is_initialized(&self) -> bool {
    self.state != PoolState::Uninitialized
  }
}

///
/// Pack trait
///
impl Pack for ExoticPool {
  // Fixed length
  const LEN: usize = 257;
  // Unpack data from [u8] to the data struct
  fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
    msg!("Read exotic pool data");
    let src = array_ref![src, 0, 257];
    let (
      owner,
      state,
      mint_lpt,
      taxman,
      mint_a,
      treasury_a,
      reserve_a,
      mint_b,
      treasury_b,
      reserve_b,
      fee_ratio,
      tax_ratio,
    ) = array_refs![src, 32, 1, 32, 32, 32, 32, 8, 32, 32, 8, 8, 8];
    Ok(ExoticPool {
      owner: Pubkey::new_from_array(*owner),
      state: PoolState::try_from_primitive(state[0]).or(Err(ProgramError::InvalidAccountData))?,
      mint_lpt: Pubkey::new_from_array(*mint_lpt),
      taxman: Pubkey::new_from_array(*taxman),
      mint_a: Pubkey::new_from_array(*mint_a),
      treasury_a: Pubkey::new_from_array(*treasury_a),
      reserve_a: u64::from_le_bytes(*reserve_a),
      mint_b: Pubkey::new_from_array(*mint_b),
      treasury_b: Pubkey::new_from_array(*treasury_b),
      reserve_b: u64::from_le_bytes(*reserve_b),
      fee_ratio: u64::from_le_bytes(*fee_ratio),
      tax_ratio: u64::from_le_bytes(*tax_ratio),
    })
  }
  // Pack data from the data struct to [u8]
  fn pack_into_slice(&self, dst: &mut [u8]) {
    msg!("Write exotic pool data");
    let dst = array_mut_ref![dst, 0, 257];
    let (
      dst_owner,
      dst_state,
      dst_mint_lpt,
      dst_taxman,
      dst_mint_a,
      dst_treasury_a,
      dst_reserve_a,
      dst_mint_b,
      dst_treasury_b,
      dst_reserve_b,
      dst_fee_ratio,
      dst_tax_ratio,
    ) = mut_array_refs![dst, 32, 1, 32, 32, 32, 32, 8, 32, 32, 8, 8, 8];
    let &ExoticPool {
      ref owner,
      state,
      ref mint_lpt,
      ref taxman,
      ref mint_a,
      ref treasury_a,
      reserve_a,
      ref mint_b,
      ref treasury_b,
      reserve_b,
      fee_ratio,
      tax_ratio,
    } = self;
    dst_owner.copy_from_slice(owner.as_ref());
    *dst_state = [state as u8];
    dst_mint_lpt.copy_from_slice(mint_lpt.as_ref());
    dst_taxman.copy_from_slice(taxman.as_ref());
    dst_mint_a.copy_from_slice(mint_a.as_ref());
    dst_treasury_a.copy_from_slice(treasury_a.as_ref());
    *dst_reserve_a = reserve_a.to_le_bytes();
    dst_mint_b.copy_from_slice(mint_b.as_ref());
    dst_treasury_b.copy_from_slice(treasury_b.as_ref());
    *dst_reserve_b = reserve_b.to_le_bytes();
    *dst_fee_ratio = fee_ratio.to_le_bytes();
    *dst_tax_ratio = tax_ratio.to_le_bytes();
  }
}
