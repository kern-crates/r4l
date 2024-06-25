use crate::driver::DeviceDriver;

pub const MDIO_DEVICE_IS_PHY: u32 = 0x80000000;

pub struct MdioDriverCommon {
    device_driver: DeviceDriver,
    flags: u32,
}

impl MdioDriverCommon {
    fn new(flags:u32, drv: DeviceDriver) -> Self {
        MdioDriverCommon{device_driver: drv, flags}
    }
}


