use std::ops::{BitAnd, Deref, Sub};

pub trait PowerOfTwo {
    fn is_power_of_two(&self) -> bool;
}

impl PowerOfTwo for u32 {
    fn is_power_of_two(&self) -> bool {
        (*self).is_power_of_two()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pow2<T>(T);

impl<T: PowerOfTwo> Pow2<T> {
    pub fn new(val: T) -> Option<Self> {
        if val.is_power_of_two() {
            Some(Self(val))
        } else {
            None
        }
    }
}

impl<T> Deref for Pow2<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Copy + PowerOfTwo + Sub<T, Output = T> + From<u8>> Pow2<T> {
    fn mask(&self) -> T {
        (*self).sub(1u8.into())
    }
}

pub trait FastRem<Rhs> {
    type Output;
    fn fast_rem(self, rhs: Rhs) -> Self::Output;
}

impl<T: Copy + PowerOfTwo + Sub<T, Output = T> + From<u8> + BitAnd<Output = T>> FastRem<Pow2<T>>
    for T
{
    type Output = T;
    fn fast_rem(self, rhs: Pow2<T>) -> Self::Output {
        self & rhs.mask()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pow2_wrapper() {
        let p = Pow2::new(8u32).expect("8 is a power of 2");

        // Test Deref
        assert_eq!(*p, 8);

        // Test Rem operator (fast modular math)
        assert_eq!(9.fast_rem(p), 1);
        assert_eq!(16.fast_rem(p), 0);
        assert_eq!(15.fast_rem(p), 7);

        // Test invalid creation
        assert!(Pow2::new(7u32).is_none());
        assert!(Pow2::new(0u32).is_none());
    }
}
