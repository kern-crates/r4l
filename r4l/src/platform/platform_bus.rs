use super::{PlatformDevice, PlatformDriver};
use alloc::collections::VecDeque;

use crate::bus::BusType;
use crate::device::DeviceOps;
use crate::error::*;
use crate::of::DeviceId::Compatible;
use crate::pr_info;
use crate::prelude::Vec;
use crate::sync::{Arc, Mutex};

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

    fn bus_driver_match(&self, pdrv: Self::Driver) -> Vec<Self::Device> {
        let mut matched_pdev = Vec::new();
        let table = pdrv
            .lock()
            .id_table()
            .expect("platform driver not define Compatible Table");
        for dev in self.devices.iter() {
            for id in table {
                match id {
                    Compatible(id) => {
                        if dev.lock().compatible_match(id) {
                            pr_info!("driver : {} device matched", id);
                            matched_pdev.push(dev.clone());
                        }
                    }
                    _ => panic!("invalid id table"),
                }
            }
        }
        matched_pdev
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

pub fn platform_driver_register(pdrv: <PlatformBus as BusType>::Driver) -> Result {
    let mut bus = PLATFORM_BUS.lock();
    bus.add_driver(pdrv.clone())?;
    let matchde_pdev = bus.bus_driver_match(pdrv.clone());
    // before probe, unlock bus
    drop(bus);
    for pdev in matchde_pdev {
        match pdrv.lock().probe {
            Some(fn_probe) => fn_probe(pdev)?,
            None => panic!("pdev not have probe call back"),
        }
    }
    Ok(())
}
