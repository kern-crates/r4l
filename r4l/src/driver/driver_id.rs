// SPDX-License-Identifier: GPL-2.0

/// A  device id array, followed by context data.
pub struct IdArray<T, U, const N: usize> {
    ids: [T; N],
    id_infos: [Option<U>; N],
}

impl<T, U, const N: usize> IdArray<T, U, N> {
    const U_NONE: Option<U> = None;

    /// Returns the number of items in the ID table.
    pub const fn count(&self) -> usize {
        self.ids.len()
    }

    /// Creates a new instance of the array.
    ///
    /// The contents are derived from the given identifiers and context information.
    #[doc(hidden)]
    pub const fn new(ids: [T; N], id_infos: [Option<U>; N]) -> Self
    {
        Self {ids,id_infos}
    }
}

// Creates a new ID array. This is a macro so it can take as a parameter the concrete ID type in order
// to call to_rawid() on it, and still remain const. This is necessary until a new const_trait_impl
// implementation lands, since the existing implementation was removed in Rust 1.73.
#[macro_export]
#[doc(hidden)]
macro_rules! _new_id_array {
    (($($args:tt)*), $id_type:ty) => {{
        /// Creates a new instance of the array.
        ///
        /// The contents are derived from the given identifiers and context information.
        const fn new< U, const N: usize>(ids: [$id_type; N], infos: [Option<U>; N])
            -> $crate::driver::IdArray<$id_type, U, N>
        {
            $crate::driver::IdArray::<$id_type, U, N>::new(ids, infos)
        }
        new($($args)*)
    }}
}

/// Counts the number of parenthesis-delimited, comma-separated items.
///
/// # Examples
///
/// ```
/// # use kernel::count_paren_items;
///
/// assert_eq!(0, count_paren_items!());
/// assert_eq!(1, count_paren_items!((A)));
/// assert_eq!(1, count_paren_items!((A),));
/// assert_eq!(2, count_paren_items!((A), (B)));
/// assert_eq!(2, count_paren_items!((A), (B),));
/// assert_eq!(3, count_paren_items!((A), (B), (C)));
/// assert_eq!(3, count_paren_items!((A), (B), (C),));
/// ```
#[macro_export]
macro_rules! count_paren_items {
    (($($item:tt)*), $($remaining:tt)*) => { 1 + $crate::count_paren_items!($($remaining)*) };
    (($($item:tt)*)) => { 1 };
    () => { 0 };
}

/// Converts a comma-separated list of pairs into an array with the first element. That is, it
/// discards the second element of the pair.
///
/// Additionally, it automatically introduces a type if the first element is warpped in curly
/// braces, for example, if it's `{v: 10}`, it becomes `X { v: 10 }`; this is to avoid repeating
/// the type.
///
/// # Examples
///
/// ```
/// # use kernel::first_item;
///
/// #[derive(PartialEq, Debug)]
/// struct X {
///     v: u32,
/// }
///
/// assert_eq!([] as [X; 0], first_item!(X, ));
/// assert_eq!([X { v: 10 }], first_item!(X, ({ v: 10 }, Y)));
/// assert_eq!([X { v: 10 }], first_item!(X, ({ v: 10 }, Y),));
/// assert_eq!([X { v: 10 }], first_item!(X, (X { v: 10 }, Y)));
/// assert_eq!([X { v: 10 }], first_item!(X, (X { v: 10 }, Y),));
/// assert_eq!([X { v: 10 }, X { v: 20 }], first_item!(X, ({ v: 10 }, Y), ({ v: 20 }, Y)));
/// assert_eq!([X { v: 10 }, X { v: 20 }], first_item!(X, ({ v: 10 }, Y), ({ v: 20 }, Y),));
/// assert_eq!([X { v: 10 }, X { v: 20 }], first_item!(X, (X { v: 10 }, Y), (X { v: 20 }, Y)));
/// assert_eq!([X { v: 10 }, X { v: 20 }], first_item!(X, (X { v: 10 }, Y), (X { v: 20 }, Y),));
/// assert_eq!([X { v: 10 }, X { v: 20 }, X { v: 30 }],
///            first_item!(X, ({ v: 10 }, Y), ({ v: 20 }, Y), ({v: 30}, Y)));
/// assert_eq!([X { v: 10 }, X { v: 20 }, X { v: 30 }],
///            first_item!(X, ({ v: 10 }, Y), ({ v: 20 }, Y), ({v: 30}, Y),));
/// assert_eq!([X { v: 10 }, X { v: 20 }, X { v: 30 }],
///            first_item!(X, (X { v: 10 }, Y), (X { v: 20 }, Y), (X {v: 30}, Y)));
/// assert_eq!([X { v: 10 }, X { v: 20 }, X { v: 30 }],
///            first_item!(X, (X { v: 10 }, Y), (X { v: 20 }, Y), (X {v: 30}, Y),));
/// ```
#[macro_export]
macro_rules! first_item {
    ($id_type:ty, $(({$($first:tt)*}, $second:expr)),* $(,)?) => {
        {
            type IdType = $id_type;
            [$(IdType{$($first)*},)*]
        }
    };
    ($id_type:ty, $(($first:expr, $second:expr)),* $(,)?) => { [$($first,)*] };
}

/// Converts a comma-separated list of pairs into an array with the second element. That is, it
/// discards the first element of the pair.
///
/// # Examples
///
/// ```
/// # use kernel::second_item;
///
/// assert_eq!([] as [u32; 0], second_item!());
/// assert_eq!([10u32], second_item!((X, 10u32)));
/// assert_eq!([10u32], second_item!((X, 10u32),));
/// assert_eq!([10u32], second_item!(({ X }, 10u32)));
/// assert_eq!([10u32], second_item!(({ X }, 10u32),));
/// assert_eq!([10u32, 20], second_item!((X, 10u32), (X, 20)));
/// assert_eq!([10u32, 20], second_item!((X, 10u32), (X, 20),));
/// assert_eq!([10u32, 20], second_item!(({ X }, 10u32), ({ X }, 20)));
/// assert_eq!([10u32, 20], second_item!(({ X }, 10u32), ({ X }, 20),));
/// assert_eq!([10u32, 20, 30], second_item!((X, 10u32), (X, 20), (X, 30)));
/// assert_eq!([10u32, 20, 30], second_item!((X, 10u32), (X, 20), (X, 30),));
/// assert_eq!([10u32, 20, 30], second_item!(({ X }, 10u32), ({ X }, 20), ({ X }, 30)));
/// assert_eq!([10u32, 20, 30], second_item!(({ X }, 10u32), ({ X }, 20), ({ X }, 30),));
/// ```
#[macro_export]
macro_rules! second_item {
    ($(({$($first:tt)*}, $second:expr)),* $(,)?) => { [$($second,)*] };
    ($(($first:expr, $second:expr)),* $(,)?) => { [$($second,)*] };
}

/// Defines a new constant [`IdArray`] with a concise syntax.
///
/// It is meant to be used by buses and subsystems to create a similar macro with their device id
/// type already specified, i.e., with fewer parameters to the end user.
///
/// # Examples
///
// TODO: Exported but not usable by kernel modules (requires `const_trait_impl`).
/// ```ignore
/// #![feature(const_trait_impl)]
/// # use kernel::{define_id_array, driver::RawDeviceId};
///
/// #[derive(Copy, Clone)]
/// struct Id(u32);
///
/// // SAFETY: `ZERO` is all zeroes and `to_rawid` stores `offset` as the second element of the raw
/// // device id pair.
/// unsafe impl const RawDeviceId for Id {
///     type RawType = (u64, isize);
///     const ZERO: Self::RawType = (0, 0);
///     fn to_rawid(&self, offset: isize) -> Self::RawType {
///         (self.0 as u64 + 1, offset)
///     }
/// }
///
/// define_id_array!(A1, Id, (), []);
/// define_id_array!(A2, Id, &'static [u8], [(Id(10), None)]);
/// define_id_array!(A3, Id, &'static [u8], [(Id(10), Some(b"id1")), ]);
/// define_id_array!(A4, Id, &'static [u8], [(Id(10), Some(b"id1")), (Id(20), Some(b"id2"))]);
/// define_id_array!(A5, Id, &'static [u8], [(Id(10), Some(b"id1")), (Id(20), Some(b"id2")), ]);
/// define_id_array!(A6, Id, &'static [u8], [(Id(10), None), (Id(20), Some(b"id2")), ]);
/// define_id_array!(A7, Id, &'static [u8], [(Id(10), Some(b"id1")), (Id(20), None), ]);
/// define_id_array!(A8, Id, &'static [u8], [(Id(10), None), (Id(20), None), ]);
///
/// // Within a bus driver:
/// driver_id_table!(BUS_ID_TABLE, Id, &'static [u8], A1);
/// // At the top level:
/// module_id_table!(MODULE_ID_TABLE, "mybus", Id, A1);
/// ```
#[macro_export]
macro_rules! define_id_array {
    ($table_name:ident, $id_type:ty, $data_type:ty, [ $($t:tt)* ]) => {
        const $table_name:
            $crate::driver::IdArray<$id_type, $data_type, { $crate::count_paren_items!($($t)*) }> =
                $crate::_new_id_array!((
                    $crate::first_item!($id_type, $($t)*), $crate::second_item!($($t)*)), $id_type);
    };
}

/// Declares an [`IdArray`] as an [`IdTable`] for a bus driver with a concise syntax.
///
/// On rust os it's the same as IdArray
/// # Examples
///
/// ```ignore
/// #![feature(const_trait_impl)]
/// # use kernel::{driver_id_table};
/// driver_id_table!(BUS_ID_TABLE, Id, &'static [u8], MY_ID_ARRAY);
/// ```
#[macro_export]
macro_rules! driver_id_table {
    ($table_name:ident, $id_type:ty, $data_type:ty, $target:expr) => {
        const OF_DEVICE_ID_TABLE_SIZE: usize = $target.count();
        const $table_name: Option<&'static $crate::driver::IdArray<$id_type, $data_type, {$target.count()}>> 
            = Some(&$target);
    };
}

/// On rust os do nothing
/// ```
#[macro_export]
macro_rules! module_id_table {
    ($item_name:ident, $table_type:literal, $id_type:ty, $table_name:ident) => {
    };
}


