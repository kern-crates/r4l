// SPDX-License-Identifier: GPL-2.0

//! Crate for all kernel procedural macros.

use proc_macro::TokenStream;

mod helpers;
mod module;
/// Declares a kernel module.
///
/// Rust os now  not support mod install
/// This is used to adapt r4l, mod init would registered to driver init
///
/// # Examples
///
/// ```ignore
/// use r4l::prelude::*;
///
/// module!{
///     type: MyModule,
///     name: "my_kernel_module",
///     author: "Rust for Linux Contributors",
///     description: "My very own kernel module!",
///     license: "GPL",
/// }
///
/// struct MyModule;
///
/// impl r4l::Module for MyModule {
///     fn init() -> Result<Self> {
///         // If the parameter is writeable, then the kparam lock must be
///         // taken to read the parameter:
///         {
///             let lock = THIS_MODULE.kernel_param_lock();
///             pr_info!("i32 param is:  {}\n", writeable_i32.read(&lock));
///         }
///         // If the parameter is read only, it can be read without locking
///         // the kernel parameters:
///         pr_info!("i32 param is:  {}\n", my_i32.read());
///         Ok(Self)
///     }
/// }
/// ```
///
/// # Supported argument types
///   - `type`: type which implements the [`Module`] trait (required).
///   - `name`: byte array of the name of the kernel module (required).
///   - `author`: byte array of the author of the kernel module.
///   - `description`: byte array of the description of the kernel module.
///   - `license`: byte array of the license of the kernel module (required).
///   - `alias`: byte array of alias name of the kernel module.
#[proc_macro]
pub fn module(ts: TokenStream) -> TokenStream {
    module::module(ts)
}

