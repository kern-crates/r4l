// SPDX-License-Identifier: GPL-2.0

//! Generic devices that are part of the kernel's driver model.
//!
//! C header: [`include/linux/device.h`](../../../../include/linux/device.h)
//!

use crate::prelude::Box;
use core::any::Any;
use of::OfNode;

pub struct Device {
    of_node: OfNode<'static>,
    drv_data: Option<Box<dyn Any>>,
}

impl Device {
    pub const fn new(of_node: OfNode<'static>) -> Self {
        Device {
            of_node,
            drv_data: None,
        }
    }

    pub fn set_drv_data<T: Any + 'static>(&mut self, drv_data: T) {
        self.drv_data = Some(Box::new(drv_data));
    }

    pub fn get_drv_data<T: Any>(&self) -> Option<&T> {
        self.drv_data.as_ref()?.downcast_ref::<T>()
    }
}

pub trait DeviceOps {
    fn set_drv_data<T: Any + 'static>(&mut self, drv_data: T);
    fn get_drv_data<T: Any>(&self) -> Option<&T>;
}
