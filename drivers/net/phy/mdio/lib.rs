use core::mem::MaybeUninit;
use kernel::bus::{BusType, BusDriver};
use kernel::prelude::*;

struct MdioBusDriver;

#[vtable]
impl BusDriver for MdioBusDriver {
    const NAME: &'static str = "mdio_bus";

    fn bus_match(&self, drv: Driver, dev: Device) -> bool {
        true
    }
}

kernel::bustype_declare! {
    driver: MdioBusDriver,
}
