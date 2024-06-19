//! Defines the R4L str mod.
//!
//! As a data type definition, str may differ on OS implemented in different languages;
//! different OSs should not differ in the same language.
//! Therefore, we use c_os rust_os feature to distinguish them.
//!
#[cfg(feature = "c_lang_os")]
mod str{
}

#[cfg(feature = "rust_os")]
mod str{
    /// Rust Os CStr is equal a &'static str. do nothing
    pub struct CStr {
        inner: &'static str, 
    }
    
    impl CStr {
        pub fn new(inner: &'static str) -> Self {
            CStr{inner}
        }
    }
    
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
                CStr::new($str)
            };
        };
    }
}

pub use str::*;

