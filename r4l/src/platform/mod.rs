// SPDX-License-Identifier: GPL-2.0

//! Platform devices and drivers.

mod platform_bus;
mod platform_dev;
mod platform_drv;

pub use platform_bus::*;
pub use platform_dev::*;
pub use platform_drv::*;
