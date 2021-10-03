use num_enum::TryFromPrimitive;
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

///
/// Pool state
///
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, TryFromPrimitive)]
pub enum PoolState {
  Uninitialized,
  Initialized,
  Frozen,
}
impl Default for PoolState {
  fn default() -> Self {
    PoolState::Uninitialized
  }
}

///
/// Admin trait
///
pub trait Admin {
  // True if frozen else False
  fn is_frozen(&self) -> bool;
  // Terminate if not owner
  fn is_owner(&self, expected_owner: Pubkey) -> Result<(), ProgramError>;
}

///
/// Pool trait
///
pub trait Liquidity {
  // Code (0: pool A, 1: pool B), Reserve
  fn get_reserve(&self, mint: &Pubkey) -> Option<(u8, u64)>;
  // Pricing curve
  fn curve(&self, bid_amount: u64, bid_mint: &Pubkey, ask_mint: &Pubkey)
    -> Option<(u64, u64, u64)>;
  // Fee
  fn fee(&self, ask_amount: u64) -> Option<(u64, u64, u64)>;
  // Add liquidity
  fn deposit(
    &self,
    delta_a: u64,
    delta_b: u64,
    liquidity: u64,
  ) -> Option<(u64, u64, u64, u64, u64, u64)>;
  // Remove liquidity
  fn withdraw(&self, lpt: u64, liquidity: u64) -> Option<(u64, u64, u64, u64, u64, u64)>;
}
