// SPDX-License-Identifier: GPL-2.0

//! Platform devices and drivers.
//!
//! Also called `platdev`, `pdev`.
//!
//! C header: [`include/linux/platform_device.h`](../../../../include/linux/platform_device.h)

use crate::{
    driver,
    error::*,
    of,
    str::CStr,
    ThisModule,
};

mod platform_dev;
mod platform_drv;

pub use platform_dev::*;
pub use platform_drv::*;


