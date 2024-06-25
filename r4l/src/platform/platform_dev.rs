// SPDX-License-Identifier: GPL-2.0

//! A platform device.
//!
//! # Invariants
//!
//! The field `ptr` is non-null and valid for the lifetime of the object.
//!

use crate::device::Device;

pub struct PlatformDevice<T> {
    id: i32,
    dev: Device<T>,
}

impl <T> PlatformDevice<T> {
    /// Returns id of the platform device.
    pub fn id(&self) -> i32 {
        self.id
    }

    #[inline]
    pub fn get_drv_data(&mut self) -> &mut T {
        self.dev.get_drv_data()
    }

    #[inline]
    pub fn set_drv_data(&mut self, data:T) {
        self.dev.set_drv_data(data)
    }
}

