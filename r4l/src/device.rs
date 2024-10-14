// SPDX-License-Identifier: GPL-2.0

//! Generic devices that are part of the kernel's driver model.
//!
//! C header: [`include/linux/device.h`](../../../../include/linux/device.h)
//!

use crate::pr_info;
use crate::prelude::*;
use crate::platform::PlatformDevice;
use core::any::Any;
use of::OfNode;

pub struct Device {
    of_node: OfNode<'static>,
    // Driver matched the first device compatiable
    drv_matched: Option<&'static str>,
    drv_data: Option<Box<dyn Any>>,
}

impl Device {
    pub const fn new(of_node: OfNode<'static>) -> Self {
        Device {
            of_node,
            drv_data: None,
            drv_matched: None,
        }
    }

    pub fn get_resource(&self, index: usize) -> Result<usize> {
        crate::of::of_membase_resource_get(self.of_node, index)
    }

    pub fn irq_resource(&self, index: usize) -> Result<u32> {
        crate::of::of_irq_get(self.of_node, index)
    }

    pub fn set_drv_data<T: Any + 'static>(&mut self, drv_data: T) {
        self.drv_data = Some(Box::new(drv_data));
    }

    pub fn get_drv_data<T: Any>(&self) -> Option<&T> {
        self.drv_data.as_ref()?.downcast_ref::<T>()
    }

    pub fn compatible_match(&self, compatible: &'static str) -> bool {
        match self.of_node.compatible() {
            Some(n) => n.all().find(|one| *one == compatible).is_some(),
            None => false,
        }
    }

    pub fn device_property_read_u32(&self, propname: &'static CStr) -> Result<u32> {
        let res = of::of_property_read_u32(self.of_node, propname, 0);
        match res {
            Some(val) => { Ok(val)}
            None => { Err(EINVAL) }
        }
    }

    pub fn from_dev(pdev: &PlatformDevice) -> &Self {
        pdev.get_device()
    }
}

pub trait DeviceOps {
    fn set_drv_data<T: Any + 'static>(&mut self, drv_data: T);
    fn get_drv_data<T: Any>(&self) -> Option<&T>;
    fn compatible_match(&self, compatible: &'static str) -> bool;
}
