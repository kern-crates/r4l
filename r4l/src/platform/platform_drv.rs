// SPDX-License-Identifier: GPL-2.0

use super::{platform_driver_register, PlatformDevice};
use crate::{
    device::DeviceOps, driver, driver::IdArray, driver::IdTable, error::*, of, prelude::*,
    sync::Arc, sync::Mutex,
};

type PlatformIdTable = &'static [of::DeviceId];

pub struct PlatformDriver {
    driver: driver::DeviceDriver,
    pub probe: Option<fn(dev: Arc<Mutex<PlatformDevice>>) -> Result>,
    remove: Option<fn(dev: &mut PlatformDevice) -> Result>,
    id_table: Option<PlatformIdTable>,
}

impl Default for PlatformDriver {
    fn default() -> Self {
        let mut s = ::core::mem::MaybeUninit::<Self>::uninit();
        unsafe {
            ::core::ptr::write_bytes(s.as_mut_ptr(), 0, 1);
            s.assume_init()
        }
    }
}

impl PlatformDriver {
    fn init(
        &mut self,
        probe: fn(dev: Arc<Mutex<PlatformDevice>>) -> Result,
        remove: fn(dev: &mut PlatformDevice) -> Result,
        id_table: Option<PlatformIdTable>,
    ) {
        self.probe = Some(probe);
        self.remove = Some(remove);
        self.id_table = id_table;
    }

    fn register(this: Arc<Self>, name: &'static CStr, module: &'static ThisModule) -> Result {
        Ok(())
    }

    fn unregister(&mut self) {}

    pub fn id_table(&self) -> Option<PlatformIdTable> {
        self.id_table
    }
}

/// A platform driver.
pub trait Driver
where
    Self: 'static,
{
    /// Data stored on RawDeviceIdce by driver.
    ///
    /// Corresponds to the data set or retrieved via the kernel's
    /// `platform_{set,get}_drvdata()` functions.
    ///
    /// Require that `Data` implements `ForeignOwnable`. We guarantee to
    /// never move the underlying wrapped data structure. This allows
    type Data: Send + Sync + driver::DeviceRemoval + Clone = ();

    /// The type holding information about each device id supported by the driver.
    type IdInfo: 'static = ();

    const OF_DEVICE_ID_TABLE_SIZE: usize = 0;
    /// The table of device ids supported by the driver.
    const OF_DEVICE_ID_TABLE: Option<&'static [of::DeviceId]> = None;

    /// Platform driver probe.
    ///
    /// Called when a new platform device is added or discovered.
    /// Implementers should attempt to initialize the device here.
    fn probe(dev: &mut PlatformDevice, id_info: Option<&Self::IdInfo>) -> Result<Self::Data>;

    /// Platform driver remove.
    ///
    /// Called when a platform device is removed.
    /// Implementers should prepare the device for complete removal here.
    fn remove(_data: &Self::Data) -> Result {
        Ok(())
    }
}

/// A registration of a platform driver.
pub type Registration<T> = driver::Registration<Adapter<T>>;

/// An adapter for the registration of platform drivers.
pub struct Adapter<T: Driver>(T);

impl<T: Driver> driver::DriverOps for Adapter<T> {
    type RegType = Arc<Mutex<PlatformDriver>>;

    fn register(
        pdrv: &mut Self::RegType,
        name: &'static CStr,
        module: &'static ThisModule,
    ) -> Result {
        pdrv.lock().init(
            Self::probe_callback,
            Self::remove_callback,
            T::OF_DEVICE_ID_TABLE,
        );
        pdrv.lock().driver.init(name, module)?;
        platform_driver_register(pdrv.clone())?;
        Ok(())
    }

    fn unregister(pdrv: &mut Self::RegType) {}
}

impl<T: Driver> Adapter<T> {
    fn get_id_info(pdev: &PlatformDevice) -> Option<&'static T::IdInfo> {
        None
    }

    fn probe_callback(pdev: Arc<Mutex<PlatformDevice>>) -> Result {
        let mut pdev = pdev.lock();
        let info = Self::get_id_info(&pdev);
        let data = T::probe(&mut pdev, info)?;
        pdev.set_drv_data(data.clone());
        Ok(())
    }

    fn remove_callback(pdev: &mut PlatformDevice) -> Result {
        let data = pdev.get_drv_data::<T::Data>().unwrap();
        let ret = T::remove(data);
        <T::Data as driver::DeviceRemoval>::device_remove(data);
        ret?;
        Ok(())
    }
}

macro_rules! module_platform_device {
    ($($f:tt)*) => {
        $crate::module_driver!(<T>, $crate::platform::Adapter<T>, { $($f)* });
    };
}

/// Declares a kernel module that exposes a single platform driver.
///
/// # Examples
///
/// ```ignore
/// # use kernel::{platform, define_of_id_table, module_platform_driver};
/// #
/// struct MyDriver;
/// impl platform::Driver for MyDriver {
///     // [...]
/// #   fn probe(_dev: &mut platform::Device, _id_info: Option<&Self::IdInfo>) -> Result {
/// #       Ok(())
/// #   }
/// #   define_of_id_table! {(), [
/// #       (of::DeviceId::Compatible(b"brcm,bcm2835-rng"), None),
/// #   ]}
/// }
///
/// module_platform_driver! {
///     type: MyDriver,
///     name: "module_name",
///     author: "Author name",
///     license: "GPL",
/// }
/// ```
#[macro_export]
macro_rules! module_platform_driver {
    ($($f:tt)*) => {
        $crate::module_driver!(<T>, $crate::platform::Adapter<T>, { $($f)* });
    };
}
