//! Os adapt for net

cfg_if::cfg_if! {
    if #[cfg(feature = "rust_os")] {
        mod rust_os;
        pub use self::rust_os::*;
    }
}

