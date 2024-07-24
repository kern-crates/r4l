// SPDX-License-Identifier: GPL-2.0

//! A platform device.
use crate::device;
use core::any::Any;
use of::OfNode;

use crate::error::Result;

pub struct PlatformDevice {
    device: device::Device,
}

impl PlatformDevice {
    pub const fn new(of_node: OfNode<'static>) -> Self {
        PlatformDevice {
            device: device::Device::new(of_node),
        }
    }
}

impl PlatformDevice {
    /// Returns irq of the platform device.
    pub fn irq_resource(&self, index: u32) -> Result<i32> {
        self.device.irq_resource(index)
    }
}

impl device::DeviceOps for PlatformDevice {
    fn set_drv_data<T: Any + 'static>(&mut self, drv_data: T) {
        self.device.set_drv_data(drv_data);
    }

    fn get_drv_data<T: Any>(&self) -> Option<&T> {
        self.device.get_drv_data::<T>()
    }

    fn compatible_match(&self, compatible: &'static str) -> bool {
        self.device.compatible_match(compatible)
    }
}
