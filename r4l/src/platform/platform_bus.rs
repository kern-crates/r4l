use super::{PlatformDevice, PlatformDriver};
use crate::bus::BusType;
use crate::error::*;
use crate::sync::{Arc, Mutex};
use alloc::collections::VecDeque;

pub struct PlatformBus {
    devices: VecDeque<Arc<Mutex<PlatformDevice>>>,
    drivers: VecDeque<Arc<Mutex<PlatformDriver>>>,
}

impl PlatformBus {
    const fn new() -> Self {
        PlatformBus {
            devices: VecDeque::new(),
            drivers: VecDeque::new(),
        }
    }
}

unsafe impl Send for PlatformBus {}
unsafe impl Sync for PlatformBus {}

impl BusType for PlatformBus {
    const NAME: &'static str = "platform";
    type Device = Arc<Mutex<PlatformDevice>>;
    type Driver = Arc<Mutex<PlatformDriver>>;

    fn bus_match(&self, device: Self::Device, driver: Self::Driver) -> bool {
        true
    }

    fn add_device(&mut self, device: Self::Device) -> Result {
        self.devices.push_back(device);
        Ok(())
    }

    fn add_driver(&mut self, driver: Self::Driver) -> Result {
        self.drivers.push_back(driver);
        Ok(())
    }
}

static PLATFORM_BUS: Mutex<PlatformBus> = Mutex::new(PlatformBus::new());

pub fn platform_device_register(device: <PlatformBus as BusType>::Device) -> Result {
    PLATFORM_BUS.lock().add_device(device)?;
    Ok(())
}

pub fn platform_driver_register(driver: <PlatformBus as BusType>::Driver) -> Result {
    let mut bus = PLATFORM_BUS.lock();
    bus.add_driver(driver)?;
    Ok(())
}
