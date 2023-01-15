pub trait NumExt {
    /// Returns the base 2 logarithm of the number, rounding up to the next integer.
    fn ilog2_ceil(self) -> Self;
}

impl NumExt for usize {
    #[inline]
    fn ilog2_ceil(self) -> Self {
        (self as f64).log2().ceil() as Self
    }
}

#[cfg(test)]
mod tests {
    use super::NumExt;

    #[test]
    fn test_ilog2() {
        assert_eq!(2_usize.ilog2_ceil(), 1);
        assert_eq!(3_usize.ilog2_ceil(), 2);
        assert_eq!(4_usize.ilog2_ceil(), 2);
        assert_eq!(5_usize.ilog2_ceil(), 3);
        assert_eq!(8_usize.ilog2_ceil(), 3);
        assert_eq!(9_usize.ilog2_ceil(), 4);
        assert_eq!(16_usize.ilog2_ceil(), 4);
        assert_eq!(17_usize.ilog2_ceil(), 5);
        assert_eq!(32_usize.ilog2_ceil(), 5);
    }
}
