// SPDX-License-Identifier: GPL-2.0

//! A platform device.
use crate::{device, io};
use core::any::{self, Any};
use of_fdt::OfNode;

use crate::error::Result;
use crate::sync::{Arc, Mutex};
use crate::i2c::Box;

use super::{I2cAdapter,I2cAlgo };

pub struct I2cClient {
    device: device::Device,
    adapter: I2cAdapter,
    data: Option<Box<dyn Any>>,
}

impl I2cClient {
    pub const fn new(of_node: OfNode<'static>, adpt: I2cAdapter) -> Self {
        I2cClient {
            device: device::Device::new(of_node),
            adapter: adpt,
            data: None,
        }
    }

    /// Returns irq of the i2c device.
    pub fn irq_resource(&self, index: usize) -> Result<u32> {
        self.device.irq_resource(index)
    }

    /// Return ioremap ptr
    pub fn ioremap_resource(&self, index: usize) -> Result<usize>{
        let addr = self.device.get_resource(index)?;
        Ok(io::ioremap(addr))
    }

    /// get device
    pub fn get_device(&self) -> &device::Device {
        &self.device
    }

    /// get adapter
    pub fn get_adapter(&self) -> &I2cAdapter {
        &self.adapter
    }

    /// set adapter data
    pub fn set_adpt_data<T: Any + 'static + Clone>(&mut self, adpt_data: T) {
        self.data = Some(Box::new(adpt_data));
    }

    /// get adapter data
    pub fn get_adpt_data<T: Any>(&self) -> Option<&T> {
        self.data.as_ref()?.downcast_ref::<T>()
    }
}

impl device::DeviceOps for I2cClient {
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
