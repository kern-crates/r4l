use crate::device::Device;

/// MdioDevice which contains a original device
pub struct MdioDevice {
    device: Device,
    flags: u32,
    mii_bus: Arc<MiiBus>,
    // Bus address of the MDIO device (0-31)
    addr: u32,
    bus_match: Option<fn(d:Device, r: Driver) -> bool>,
}

impl MdioDevice {
    fn new(mii_bus: Arc<MiiBus>, addr: u32) -> Self {
        let device = Device::new()
        Self {

            mii_bus,
            addr,
        }
    }

    fn remove(&mut self){

    }

    fn free(&mut self) {
    }
}


