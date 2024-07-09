// SPDX-License-Identifier: GPL-2.0

//! Generic devices that are part of the kernel's driver model.
//!
//! C header: [`include/linux/device.h`](../../../../include/linux/device.h)
//!

use crate::pr_info;
use crate::prelude::Box;
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
}

pub trait DeviceOps {
    fn set_drv_data<T: Any + 'static>(&mut self, drv_data: T);
    fn get_drv_data<T: Any>(&self) -> Option<&T>;
    fn compatible_match(&self, compatible: &'static str) -> bool;
}
