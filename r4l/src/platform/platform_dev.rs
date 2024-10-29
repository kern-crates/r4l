// SPDX-License-Identifier: GPL-2.0

//! A platform device.
use crate::{device, io};
use core::any::Any;
use of_fdt::OfNode;

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
    pub fn irq_resource(&self, index: usize) -> Result<u32> {
        self.device.irq_resource(index)
    }

    /// Return ioremap ptr
    pub fn ioremap_resource(&self, index: usize) -> Result<usize>{
        let addr = self.device.get_resource(index)?;
        Ok(io::ioremap(addr))
    }

    /// get device
    pub fn get_device(&self) -> device::Device {
       self.device.clone()
    }

    /// get node
    pub fn of_node(&self) -> OfNode<'static> {
        self.device.get_node()
    }

}

impl device::DeviceOps for PlatformDevice {
    fn set_drv_data<T: Any + 'static + Clone>(&mut self, drv_data: T) {
        self.device.set_drv_data(drv_data);
    }

    fn get_drv_data<T: Any>(&self) -> Option<&T> {
        self.device.get_drv_data::<T>()
    }

    fn compatible_match(&self, compatible: &'static str) -> bool {
        self.device.compatible_match(compatible)
    }
}
