use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey};

///
/// Pool operation trait
///
pub trait Operation {
  // True if frozen else False
  fn is_frozen(&self) -> bool;
  // Verify pool owner
  fn is_owner(&self, expected_owner: Pubkey) -> ProgramResult;
}

///
/// Pool exchange trait
///
pub trait Exchange {
  // Get code () for A, 1 for B) and reserve
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
