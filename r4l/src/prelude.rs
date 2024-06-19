// SPDX-License-Identifier: GPL-2.0

//! The `kernel` prelude.
//!
//! These are the most common items used by Rust code in the kernel,
//! intended to be imported by all Rust code, for convenience.
//!
//! # Examples
//!
//! ```
//! use r4l::prelude::*;
//! ```

#[doc(no_inline)]
pub use alloc::{boxed::Box, vec::Vec};

#[doc(no_inline)]
pub use macros::module;
pub use super::{pr_alert, pr_crit, pr_debug, pr_emerg, pr_err, pr_info, pr_notice, pr_warn};
pub use super::ThisModule;
pub use super::error::{code::*, Error, Result};
