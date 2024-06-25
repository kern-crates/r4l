// SPDX-License-Identifier: GPL-2.0

use core::ops::Deref;

use crate::prelude::*;
use crate::{
    sync::Arc,
    c_str,
};

use super::IdArray;

pub struct DeviceDriver {
    name: &'static str,
    //bus: Option<Arc<BusType>>,
    owner: &'static ThisModule,
}

impl Default for DeviceDriver {
    fn default() -> Self {
        let mut s = ::core::mem::MaybeUninit::<Self>::uninit();
        unsafe {
            ::core::ptr::write_bytes(s.as_mut_ptr(), 0, 1);
            s.assume_init()
        }
    }
}

impl DeviceDriver {
    pub fn register(&mut self,
      name: &'static str,
      owner: &'static ThisModule) -> Result {
        self.name = name;
        self.owner = owner;
        Ok(())
    }
}

/// A subsystem (e.g., PCI, Platform, Amba, etc.) that allows drivers to be written for it.
pub trait DriverOps {
    /// The type that holds information about the registration. This is typically a struct defined
    /// by the C portion of the kernel.
    type RegType: Default;

    /// Registers a driver.
    ///
    /// # Safety
    ///
    /// `reg` must point to valid, initialised, and writable memory. It may be modified by this
    /// function to hold registration state.
    ///
    /// On success, `reg` must remain pinned and valid until the matching call to
    /// [`DriverOps::unregister`].
    fn register(reg: &mut Self::RegType,
        name: &'static CStr,
        module: &'static ThisModule) -> Result;

    /// Unregisters a driver previously registered with [`DriverOps::register`].
    ///
    /// # Safety
    ///
    /// `reg` must point to valid writable memory, initialised by a previous successful call to
    /// [`DriverOps::register`].
    fn unregister(reg: &mut Self::RegType);
}

/// The registration of a driver.
pub struct Registration<T: DriverOps> {
    is_registered: bool,
    concrete_reg: T::RegType,
}

// SAFETY: `Registration` has no fields or methods accessible via `&Registration`, so it is safe to
// share references to it with multiple threads as nothing can be done.
unsafe impl<T: DriverOps> Sync for Registration<T> {}
unsafe impl<T: DriverOps> Send for Registration<T> {}

impl<T: DriverOps> Registration<T> {
    /// Creates a new instance of the registration object.
    pub fn new() -> Self {
        Self {
            is_registered: false,
            concrete_reg: T::RegType::default(),
        }
    }

    /// Allocates a pinned registration object and registers it.
    ///
    /// Returns a pinned heap-allocated representation of the registration.
    pub fn new_pinned(name: &'static CStr, module: &'static ThisModule) -> Result<Box<Self>> {
        let mut reg = Box::new(Self::new());
        reg.register(name, module)?;
        Ok(reg)
    }

    /// Registers a driver with its subsystem.
    ///
    /// It must be pinned because the memory block that represents the registration is potentially
    /// self-referential.
    pub fn register(&mut self, name: &'static CStr, module: &'static ThisModule) -> Result {
        if self.is_registered {
            return Err(EINVAL);
        }
        T::register(&mut self.concrete_reg, name, module)?;
        self.is_registered = true;
        Ok(())
    }
}

impl<T: DriverOps> Default for Registration<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: DriverOps> Drop for Registration<T> {
    fn drop(&mut self) {
        if self.is_registered {
            T::unregister(&mut self.concrete_reg);
        }
    }
}

/// Custom code within device removal.
pub trait DeviceRemoval {
    /// Cleans resources up when the device is removed.
    ///
    /// This is called when a device is removed and offers implementers the chance to run some code
    /// that cleans state up.
    fn device_remove(&self);
}

impl DeviceRemoval for () {
    fn device_remove(&self) {}
}

impl<T: DeviceRemoval> DeviceRemoval for Arc<T> {
    fn device_remove(&self) {
        self.deref().device_remove();
    }
}

impl<T: DeviceRemoval> DeviceRemoval for Box<T> {
    fn device_remove(&self) {
        self.deref().device_remove();
    }
}


/// A kernel module that only registers the given driver on init.
///
/// This is a helper struct to make it easier to define single-functionality modules, in this case,
/// modules that offer a single driver.
pub struct Module<T: DriverOps> {
    _driver: Box<Registration<T>>,
}

impl<T: DriverOps> crate::Module for Module<T> {
    fn init(module: &'static ThisModule) -> Result<Self> {
        Ok(Self {
            _driver: Registration::new_pinned(c_str!("no-name"), module)?,
        })
    }
}

/// Declares a kernel module that exposes a single driver.
///
/// It is meant to be used as a helper by other subsystems so they can more easily expose their own
/// macros.
#[macro_export]
macro_rules! module_driver {
    (<$gen_type:ident>, $driver_ops:ty, { type: $type:ty, $($f:tt)* }) => {
        type Ops<$gen_type> = $driver_ops;
        type ModuleType = $crate::driver::Module<Ops<$type>>;
        $crate::prelude::module! {
            type: ModuleType,
            $($f)*
        }
    }
}
