// SPDX-License-Identifier: GPL-2.0

//! Crate for all kernel procedural macros.

use proc_macro::TokenStream;

mod helpers;
mod module;
mod vtable;
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

/// Declares or implements a vtable trait.
///
/// Linux's use of pure vtables is very close to Rust traits, but they differ
/// in how unimplemented functions are represented. In Rust, traits can provide
/// default implementation for all non-required methods (and the default
/// implementation could just return `Error::EINVAL`); Linux typically use C
/// `NULL` pointers to represent these functions.
///
/// This attribute closes that gap. A trait can be annotated with the
/// `#[vtable]` attribute. Implementers of the trait will then also have to
/// annotate the trait with `#[vtable]`. This attribute generates a `HAS_*`
/// associated constant bool for each method in the trait that is set to true if
/// the implementer has overridden the associated method.
///
/// For a trait method to be optional, it must have a default implementation.
/// This is also the case for traits annotated with `#[vtable]`, but in this
/// case the default implementation will never be executed. The reason for this
/// is that the functions will be called through function pointers installed in
/// C side vtables. When an optional method is not implemented on a `#[vtable]`
/// trait, a NULL entry is installed in the vtable. Thus the default
/// implementation is never called. Since these traits are not designed to be
/// used on the Rust side, it should not be possible to call the default
/// implementation. This is done to ensure that we call the vtable methods
/// through the C vtable, and not through the Rust vtable. Therefore, the
/// default implementation should call `kernel::build_error`, which prevents
/// calls to this function at compile time:
///
/// ```compile_fail
/// # use kernel::error::VTABLE_DEFAULT_ERROR;
/// kernel::build_error(VTABLE_DEFAULT_ERROR)
/// ```
///
/// Note that you might need to import [`kernel::error::VTABLE_DEFAULT_ERROR`].
///
/// This macro should not be used when all functions are required.
///
/// # Examples
///
/// ```ignore
/// use kernel::error::VTABLE_DEFAULT_ERROR;
/// use kernel::prelude::*;
///
/// // Declares a `#[vtable]` trait
/// #[vtable]
/// pub trait Operations: Send + Sync + Sized {
///     fn foo(&self) -> Result<()> {
///         kernel::build_error(VTABLE_DEFAULT_ERROR)
///     }
///
///     fn bar(&self) -> Result<()> {
///         kernel::build_error(VTABLE_DEFAULT_ERROR)
///     }
/// }
///
/// struct Foo;
///
/// // Implements the `#[vtable]` trait
/// #[vtable]
/// impl Operations for Foo {
///     fn foo(&self) -> Result<()> {
/// #        Err(EINVAL)
///         // ...
///     }
/// }
///
/// assert_eq!(<Foo as Operations>::HAS_FOO, true);
/// assert_eq!(<Foo as Operations>::HAS_BAR, false);
/// ```
///
/// [`kernel::error::VTABLE_DEFAULT_ERROR`]: ../kernel/error/constant.VTABLE_DEFAULT_ERROR.html
#[proc_macro_attribute]
pub fn vtable(attr: TokenStream, ts: TokenStream) -> TokenStream {
    vtable::vtable(attr, ts)
}
