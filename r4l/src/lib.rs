//! This is an OS interface abstraction layer developed for cross-kernel drivers.
//!
//! Include:
//! - error: error type used by drivers
//! - log: log interface used by drivers

#![no_std]
#![feature(associated_type_defaults)]
#![feature(generic_const_exprs)]
#![feature(generic_const_items)]

extern crate self as kernel;

extern crate alloc;

//pub mod net;
//pub mod i2c;
mod build_error;
mod bus;
pub mod device;
pub mod driver;
pub mod error;
pub mod init;
pub mod linked_list;
pub mod of;
pub mod platform;
pub mod prelude;
pub mod print;
pub mod str;
pub mod sync;
pub mod uapi;

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
