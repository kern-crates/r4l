//! Bus
//!
//! Linux reuesed kset manage bus and bus devices and drivers.
//! Now, we don't creat file
use crate::error::*;
use crate::sync::Arc;
use core::ops::Deref;

// A bus type Trait
pub trait BusType {
    const NAME: &'static str;
    type Device = ();
    type Driver = ();

    fn bus_match(&self, device: Self::Device, driver: Self::Driver) -> bool;
    fn add_device(&mut self, device: Self::Device) -> Result;
    fn add_driver(&mut self, driver: Self::Driver) -> Result;
}
