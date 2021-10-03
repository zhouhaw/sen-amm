use num_enum::TryFromPrimitive;

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
