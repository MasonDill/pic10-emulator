use derive_more::*;

#[derive(Add, Sub, BitAnd, BitOr, Shl, Shr, Sum, Not, Into, PartialEq, PartialOrd, Eq)]
pub struct NBitNumber<const N: usize> {
    pub value: u16,
}

const fn validate_bit_width<const N: usize>() {
    assert!(N > 0 && N <= 16, "Bit width must be between 1 and 16");
}

pub trait NumberOperations<const N: usize> {
    fn get_max() -> Self;
    fn as_u16(&self) -> u16;
    fn as_usize(&self) -> usize;
    fn get(&self) -> u16;
    fn new() -> Self;
}

impl<const N: usize> NBitNumber<N> {
    pub const BITS: usize = N;

    pub const fn new(value: u16) -> Self {
        validate_bit_width::<N>();
        NBitNumber { value: value & ((1 << N) - 1) }
    }

    pub fn get(&self) -> u16 {
        self.value
    }

    /// Compares the lower bits of two NBitNumbers of potentially different sizes
    /// The comparison is done using the minimum of the two bit widths
    pub fn compare_lower_bits<const M: usize>(&self, other: &NBitNumber<M>) -> bool {
        let min_bits = std::cmp::min(N, M);
        let mask = (1 << min_bits) - 1;
        (self.value & mask) == (other.value & mask)
    }

    pub fn compare_upper_bits<const M: usize>(&self, other: &NBitNumber<M>) -> bool {
        let min_bits = std::cmp::min(N, M);
        let mask = (1 << min_bits) - 1;
        (self.value & mask) == (other.value & mask)
    }
}

impl<const N: usize> NumberOperations<N> for NBitNumber<N> {
    fn get_max() -> Self {
        validate_bit_width::<N>();
        NBitNumber::<N>::new((1 << N) - 1)
    }

    fn as_u16(&self) -> u16 {
        self.value
    }

    fn as_usize(&self) -> usize {
        self.value as usize
    }

    fn get(&self) -> u16 {
        self.value
    }

    fn new() -> Self {
        NBitNumber::<N>::new(0)
    }
}

impl<const N: usize> Clone for NBitNumber<N> {
    fn clone(&self) -> Self {
        NBitNumber::<N>::new(self.value)
    }
}

impl<const N: usize> Copy for NBitNumber<N> {}

// Type aliases for common bit widths
pub type u12 = NBitNumber<12>;
pub type u7 = NBitNumber<7>;
pub type u5 = NBitNumber<5>;
pub type u9 = NBitNumber<9>;
pub type u3 = NBitNumber<3>;
pub type u2 = NBitNumber<2>;

pub enum NBit {
    N1(NBitNumber<1>),
    N2(NBitNumber<2>),
    N3(NBitNumber<3>),
    N4(NBitNumber<4>),
    N5(NBitNumber<5>),
    N6(NBitNumber<6>),
    N7(NBitNumber<7>),
    N8(NBitNumber<8>),
    N9(NBitNumber<9>),
    N10(NBitNumber<10>),
    N11(NBitNumber<11>),
    N12(NBitNumber<12>),
    N13(NBitNumber<13>),
    N14(NBitNumber<14>),
    N15(NBitNumber<15>),
    N16(NBitNumber<16>),
}