//! Bus 
//!
//! Linux reuesed kset manage bus and bus devices and drivers.
//! Now, we don't creat file 
use crate::sync::Arc;
use core::ops::Deref;
use crate::prelude::*;
use crate::error::VTABLE_DEFAULT_ERROR;
use crate::device::Device;
use crate::driver::Driver;

use alloc::collections::VecDeque;

pub struct BusType {
    name: &'static str,
    bus_match: Option<fn(Device,Driver) -> bool>,
    probe: Option<fn(Device) -> Result>,
    device_list: VecDeque<Device>,
    driver_list: VecDeque<Driver>,
}

// A bus driver wrapper
#[vtable]
pub trait BusDriver {
    const NAME: &'static str;

    fn bus_match(&self, device: Device, driver: Driver) -> bool { 
        false
    }

    fn probe(&self, device: Device) -> Result {
        kernel::build_error(VTABLE_DEFAULT_ERROR)
    }
}

/// An adapter for the registration of a PHY driver.
struct Adapter<T: BusDriver> {
    _p: PhantomData<T>,
}

/// Driver structure for a particular PHY type.
impl<T: BusDriver> Adapter<T> {
    fn bus_match(dev: Device, driver: Driver) -> bool {
        T::bus_match(dev,driver)
    }

    fn probe(device: Device) -> Result {
        T::probe()
    }
}

/// Create a busType use BusDriver
pub const fn create_bus_type<T: BusDriver>() -> BusType {
    BusType{
        name: T::NAME,
        bus_match: if T::HAS_BUS_MATCH {
            Some(Adapter::<T>::bus_match)
        } else {
            None
        },
        probe: if T::HAS_PROBE {
            Some(Adapter::<T>::probe)
        } else {
            None
        },
        device_list: VecDeque::new(),
        driver_list: VecDeque::new(),
    }
}

/// Declares a bus type.
///
/// This creates a `struct BusType` and registers it.
///
/// # Examples
///
/// ```
/// # mod bus_sample {
/// use kernel::prelude::*;
/// use kernel::bus::BusDriver;
///
/// kernel::bustype_declare! {
///     driver: MdioBusDriver,
/// }
///
/// struct MdioBusDriver;
///
/// #[vtable]
/// impl BusDriver for MdioBusDriver {
///     const NAME: "mdio_bus",
///
///     fn bus_match(&self, device: Device, driver: Driver) -> bool {
///         ...
///     }
/// }
///
/// //now we could through bus_sample::get_bus_type() get 
/// //BusType
/// ```
///
#[macro_export]
macro_rules! bustype_declare {
    (driver: $($driver:ident)) => {
        static mut _BUS: BusType = 
            $crate::bus::create_bus_type::<$driver>();
        pub fn get_bus_type() -> &mut BusType {
            &_BUS
        }
    }
}
