//! Defines the R4L str mod.
//!
//! As a data type definition, str may differ on OS implemented in different languages;
//! different OSs should not differ in the same language.
//! Therefore, we use c_os rust_os feature to distinguish them.
//!
#[cfg(feature = "rust_os")]
mod str {
    pub type CStr = str;

    /// Creates a new [`CStr`] from a string literal.
    ///
    /// empty
    ///
    /// ```
    /// # use kernel::c_str;
    /// # use kernel::str::CStr;
    /// const MY_CSTR: &CStr = c_str!("My awesome CStr!");
    /// ```
    #[macro_export]
    macro_rules! c_str {
        ($str:expr) => {{
            const C: &'static str = $str;
            C
        }};
    }

    use alloc::string::String;
    use core::fmt;
    use crate::prelude::Error;

    pub struct CString(String);
    impl CString {
        pub fn try_from_fmt(args: fmt::Arguments<'_>) -> Result<Self, Error> {
            Ok(Self(alloc::format!("{}",args)))
        }
    }
}

pub use str::*;
