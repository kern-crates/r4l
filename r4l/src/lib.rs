//! This is an OS interface abstraction layer developed for cross-kernel drivers.
//!
//! Include:
//! - error: error type used by drivers
//! - log: log interface used by drivers

#![no_std]

extern crate self as kernel;

extern crate alloc;

//mod bus;
pub mod str;
pub mod uapi;
pub mod net;
//pub mod sync;
pub mod error;
pub mod linked_list;
pub mod init;
pub mod prelude;
pub mod print;
mod build_error;

pub use build_error::build_error;

/// The top level entrypoint to implementing a kernel module.
///
/// For any teardown or cleanup operations, your type may implement [`Drop`].
pub trait Module: Sized + Sync + Send {
    /// Called at module initialization time.
    ///
    /// Use this method to perform whatever setup or registration your module
    /// should do.
    ///
    /// Equivalent to the `module_init` macro in the C API.
    fn init(module: &'static ThisModule) -> error::Result<Self>;
}


/// Replace Linux `THIS_MODULE` in the C API.
///
pub struct ThisModule();

// SAFETY: `THIS_MODULE` may be used from all threads within a module.
unsafe impl Sync for ThisModule {}
