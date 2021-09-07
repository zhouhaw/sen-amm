const PRECISION = 1000000000000000000n;

BigInt.prototype.sqrt = function () {
  if (this < 2n) return this;
  const bits = Math.floor((this.toString(2).length + 1) / 2);
  let start = 1n << BigInt(bits - 1);
  let end = 1n << BigInt(bits + 1);
  while (start < end) {
    end = (start + end) >> 1n;
    start = this / end;
  }
  return end;
};

class AMM {
  constructor(A, B) {
    this.A = A;
    this.B = B;
  }

  checkLiquidity = (a, b) => {
    const ratio = (a * PRECISION) / b;
    const expectedRatio = (this.A * PRECISION) / this.B;
    console.log(ratio, expectedRatio);
    return ratio === expectedRatio;
  };

  swap = (amount, bid = "A", ask = "B") => {
    const prevBid = this[bid];
    this[bid] = this[bid] + amount;
    const alpha = (prevBid * PRECISION) / this[bid];
    const prevAsk = this[ask];
    this[ask] = (prevAsk * PRECISION) / (2n * PRECISION - alpha);
    return prevAsk - this[ask];
  };

  add_liquidity = (a, b) => {
    const aHat = (a * this.B - b * this.A) / (2n * (b + this.B));
    const aStar = a - aHat;
    const bHat = (a * this.B - b * this.A) / (2n * (a + this.A));
    const bStar = b + bHat;
    console.log(a - aHat, b + bHat, ((a - aHat) * PRECISION) / (b + bHat));
    console.log(
      this.A + aHat,
      this.B - bHat,
      ((this.A + aHat) * PRECISION) / (this.B - bHat)
    );
    if (!this.checkLiquidity(aStar, bStar))
      throw new Error("Assymetric deposit");
    const liquidity = (a * b).sqrt();
    return liquidity;
  };
}

const A = 1000000000n;
const B = 5000000000n;
const amm = new AMM(A, B);
// const amount = amm.swap(1000000n, "B", "A");
const lp = amm.add_liquidity(1000n, 4000n);
