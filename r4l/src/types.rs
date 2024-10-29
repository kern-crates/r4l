// SPDX-License-Identifier: GPL-2.0
//! Kernel types.

use core::ops;


/// A bitmask.
///
/// It has a restriction that all bits must be the same, except one. For example, `0b1110111` and
/// `0b1000` are acceptable masks.
#[derive(Clone, Copy)]
pub struct Bit<T> {
    index: T,
    inverted: bool,
}

/// Creates a bit mask with a single bit set.
///
/// # Examples
///
/// ```
/// # use kernel::bit;
/// let mut x = 0xfeu32;
///
/// assert_eq!(x & bit(0), 0);
/// assert_eq!(x & bit(1), 2);
/// assert_eq!(x & bit(2), 4);
/// assert_eq!(x & bit(3), 8);
///
/// x |= bit(0);
/// assert_eq!(x, 0xff);
///
/// x &= !bit(1);
/// assert_eq!(x, 0xfd);
///
/// x &= !bit(7);
/// assert_eq!(x, 0x7d);
///
/// let y: u64 = bit(34).into();
/// assert_eq!(y, 0x400000000);
///
/// assert_eq!(y | bit(35), 0xc00000000);
/// ```
pub const fn bit<T: Copy>(index: T) -> Bit<T> {
    Bit {
        index,
        inverted: false,
    }
}

impl<T: Copy> ops::Not for Bit<T> {
    type Output = Self;
    fn not(self) -> Self {
        Self {
            index: self.index,
            inverted: !self.inverted,
        }
    }
}

/// Implemented by integer types that allow counting the number of trailing zeroes.
pub trait TrailingZeros {
    /// Returns the number of trailing zeroes in the binary representation of `self`.
    fn trailing_zeros(&self) -> u32;
}

macro_rules! define_unsigned_number_traits {
    ($type_name:ty) => {
        impl TrailingZeros for $type_name {
            fn trailing_zeros(&self) -> u32 {
                <$type_name>::trailing_zeros(*self)
            }
        }

        impl<T: Copy> core::convert::From<Bit<T>> for $type_name
        where
            Self: ops::Shl<T, Output = Self> + core::convert::From<u8> + ops::Not<Output = Self>,
        {
            fn from(v: Bit<T>) -> Self {
                let c = Self::from(1u8) << v.index;
                if v.inverted {
                    !c
                } else {
                    c
                }
            }
        }

        impl<T: Copy> ops::BitAnd<Bit<T>> for $type_name
        where
            Self: ops::Shl<T, Output = Self> + core::convert::From<u8>,
        {
            type Output = Self;
            fn bitand(self, rhs: Bit<T>) -> Self::Output {
                self & Self::from(rhs)
            }
        }

        impl<T: Copy> ops::BitOr<Bit<T>> for $type_name
        where
            Self: ops::Shl<T, Output = Self> + core::convert::From<u8>,
        {
            type Output = Self;
            fn bitor(self, rhs: Bit<T>) -> Self::Output {
                self | Self::from(rhs)
            }
        }

        impl<T: Copy> ops::BitAndAssign<Bit<T>> for $type_name
        where
            Self: ops::Shl<T, Output = Self> + core::convert::From<u8>,
        {
            fn bitand_assign(&mut self, rhs: Bit<T>) {
                *self &= Self::from(rhs)
            }
        }

        impl<T: Copy> ops::BitOrAssign<Bit<T>> for $type_name
        where
            Self: ops::Shl<T, Output = Self> + core::convert::From<u8>,
        {
            fn bitor_assign(&mut self, rhs: Bit<T>) {
                *self |= Self::from(rhs)
            }
        }
    };
}

define_unsigned_number_traits!(u8);
define_unsigned_number_traits!(u16);
define_unsigned_number_traits!(u32);
define_unsigned_number_traits!(u64);
define_unsigned_number_traits!(usize);

/// Returns an iterator over the set bits of `value`.
///
/// # Examples
///
/// ```
/// use kernel::bits_iter;
///
/// let mut iter = bits_iter(5usize);
/// assert_eq!(iter.next().unwrap(), 0);
/// assert_eq!(iter.next().unwrap(), 2);
/// assert!(iter.next().is_none());
/// ```
///
/// ```
/// use kernel::bits_iter;
///
/// fn print_bits(x: usize) {
///     for bit in bits_iter(x) {
///         pr_info!("{}\n", bit);
///     }
/// }
///
/// # print_bits(42);
/// ```
#[inline]
pub fn bits_iter<T>(value: T) -> impl Iterator<Item = u32>
where
    T: core::cmp::PartialEq
        + From<u8>
        + ops::Shl<u32, Output = T>
        + ops::Not<Output = T>
        + ops::BitAndAssign
        + TrailingZeros,
{
    struct BitIterator<U> {
        value: U,
    }

    impl<U> Iterator for BitIterator<U>
    where
        U: core::cmp::PartialEq
            + From<u8>
            + ops::Shl<u32, Output = U>
            + ops::Not<Output = U>
            + ops::BitAndAssign
            + TrailingZeros,
    {
        type Item = u32;

        #[inline]
        fn next(&mut self) -> Option<u32> {
            if self.value == U::from(0u8) {
                return None;
            }
            let ret = self.value.trailing_zeros();
            self.value &= !(U::from(1u8) << ret);
            Some(ret)
        }
    }

    BitIterator { value }
}

/// A trait for boolean types.
///
/// This is meant to be used in type states to allow boolean constraints in implementation blocks.
/// In the example below, the implementation containing `MyType::set_value` could _not_ be
/// constrained to type states containing `Writable = true` if `Writable` were a constant instead
/// of a type.
///
/// # Safety
///
/// No additional implementations of [`Bool`] should be provided, as [`True`] and [`False`] are
/// already provided.
///
/// # Examples
///
/// ```
/// # use kernel::{Bool, False, True};
/// use core::marker::PhantomData;
///
/// // Type state specifies whether the type is writable.
/// trait MyTypeState {
///     type Writable: Bool;
/// }
///
/// // In state S1, the type is writable.
/// struct S1;
/// impl MyTypeState for S1 {
///     type Writable = True;
/// }
///
/// // In state S2, the type is not writable.
/// struct S2;
/// impl MyTypeState for S2 {
///     type Writable = False;
/// }
///
/// struct MyType<T: MyTypeState> {
///     value: u32,
///     _p: PhantomData<T>,
/// }
///
/// impl<T: MyTypeState> MyType<T> {
///     fn new(value: u32) -> Self {
///         Self {
///             value,
///             _p: PhantomData,
///         }
///     }
/// }
///
/// // This implementation block only applies if the type state is writable.
/// impl<T> MyType<T>
/// where
///     T: MyTypeState<Writable = True>,
/// {
///     fn set_value(&mut self, v: u32) {
///         self.value = v;
///     }
/// }
///
/// let mut x = MyType::<S1>::new(10);
/// let mut y = MyType::<S2>::new(20);
///
/// x.set_value(30);
///
/// // The code below fails to compile because `S2` is not writable.
/// // y.set_value(40);
/// ```
pub unsafe trait Bool {}

/// Represents the `true` value for types with [`Bool`] bound.
pub struct True;

// SAFETY: This is one of the only two implementations of `Bool`.
unsafe impl Bool for True {}

/// Represents the `false` value for types wth [`Bool`] bound.
pub struct False;

// SAFETY: This is one of the only two implementations of `Bool`.
unsafe impl Bool for False {}

/// Generate a mask where all bits >= `h` and <= `l` are set
///
/// This is a re-implementation in rust of `GENMASK`
pub const fn genmask(h: u32, l: u32) -> u32 {
    ((!0u32) - (1 << l) + 1) & ((!0u32) >> (32 - 1 - h))
}