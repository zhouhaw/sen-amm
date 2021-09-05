///
/// Implement square root for u128
///
pub trait Roots {
  fn sqrt(self) -> Self;
}

impl Roots for u128 {
  ///
  /// Babylonian method (with a selectively initial guesses)
  /// O(log(log(n))) for complexity
  ///
  fn sqrt(self) -> Self {
    if self < 2 {
      return self;
    }

    let bits = (128 - self.leading_zeros() + 1) / 2;
    let mut start = 1 << (bits - 1);
    let mut end = 1 << (bits + 1);
    while start < end {
      end = (start + end) / 2;
      start = self / end;
    }
    end
  }
}
