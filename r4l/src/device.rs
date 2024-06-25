// SPDX-License-Identifier: GPL-2.0

//! Generic devices that are part of the kernel's driver model.
//!
//! C header: [`include/linux/device.h`](../../../../include/linux/device.h)
//!

//use crate::bus::BusType;
use crate::Module;
use crate::driver::DeviceDriver;

pub struct Device<T> {
    //driver: Option<Arc<DeviceDriver>>,
    init_name: &'static str,
    //bus: Arc<BusType>,
    driver_data: Option<T>,
}

impl<T> Device<T> {
    pub const fn new() -> Self {
        Device {
            //driver: None, 
            //bus,
            init_name: "empty",
            driver_data: None,
        }
    }

    pub fn set_drv_data(&mut self, data:T) {
        self.driver_data = Some(data);
    }

    pub fn get_drv_data(&mut self) -> &mut T {
        self.driver_data.as_mut().expect("no drv data")
    }
}

