pub trait ToBitmask {
    /// Creates a bitmask from a number of bits stored in the variable.
    /// For example, 4 gets converted into 0b1111
    fn to_bitmask(&self) -> u64;
}

impl ToBitmask for u8 {
    fn to_bitmask(&self) -> u64 {
        if *self == 64 {
            u64::MAX
        } else {
            (1u64 << *self) - 1
        }
    }
}

impl ToBitmask for u16 {
    fn to_bitmask(&self) -> u64 {
        if *self == 64 {
            u64::MAX
        } else {
            (1u64 << *self) - 1
        }
    }
}

impl ToBitmask for u32 {
    fn to_bitmask(&self) -> u64 {
        if *self == 64 {
            u64::MAX
        } else {
            (1u64 << *self) - 1
        }
    }
}
