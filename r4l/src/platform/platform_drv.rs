// SPDX-License-Identifier: GPL-2.0

use super::PlatformDevice;
use crate::{
    of,
    driver,
    driver::IdArray,
    error::*,
    prelude::*,
};

pub struct PlatformDriver<D> {
    driver: driver::DeviceDriver,
    probe: Option<fn(dev: &mut PlatformDevice<D>) -> Result>,
    remove: Option<fn(dev: &mut PlatformDevice<D>) -> Result>,
}

impl <D> Default for PlatformDriver<D> {
    fn default() -> Self {
        let mut s = ::core::mem::MaybeUninit::<Self>::uninit();
        unsafe {
            ::core::ptr::write_bytes(s.as_mut_ptr(), 0, 1);
            s.assume_init()
        }
    }
}

impl<D> PlatformDriver<D> {
    fn init(&mut self,
        probe: fn(dev: &mut PlatformDevice<D>) -> Result, 
        remove: fn(dev: &mut PlatformDevice<D>) -> Result) 
    {
        self.probe = Some(probe);
        self.remove = Some(remove);
    }

    fn register(&mut self,
        name: &'static CStr,
        module: &'static ThisModule) -> Result {
        self.driver.register(name, module)?;
        Ok(())
    }

    fn unregister(&mut self) {
    }
}

/// A platform driver.
pub trait Driver where Self: 'static {
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
    const OF_DEVICE_ID_TABLE: Option<&'static IdArray<of::DeviceId, Self::IdInfo, {Self::OF_DEVICE_ID_TABLE_SIZE}>> 
        where [(); Self::OF_DEVICE_ID_TABLE_SIZE]: ;

    /// Platform driver probe.
    ///
    /// Called when a new platform device is added or discovered.
    /// Implementers should attempt to initialize the device here.
    fn probe(dev: &mut PlatformDevice<Self::Data>, id_info: Option<&Self::IdInfo>) -> Result<Self::Data>;

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
    type RegType = PlatformDriver<T::Data>;

    fn register(pdrv: &mut Self::RegType,
        name: &'static CStr,
        module: &'static ThisModule) -> Result {
        
        pdrv.init(Self::probe_callback, Self::remove_callback);
        pdrv.register(name, module)?;
        Ok(())
    }

    fn unregister(pdrv: &mut Self::RegType) {
        pdrv.unregister();
    }
}

impl<T: Driver> Adapter<T> {
    fn get_id_info(_dev: &PlatformDevice<T::Data>) -> Option<&'static T::IdInfo> {
        None
    }

    fn probe_callback(pdev: &mut PlatformDevice<T::Data>) -> Result {
        let info = Self::get_id_info(&pdev);
        let data = T::probe(pdev, info)?;
        pdev.set_drv_data(data.clone());
        Ok(())
    }

    fn remove_callback(pdev: &mut PlatformDevice<T::Data>) -> Result {
        let data  = pdev.get_drv_data();
        let ret = T::remove(data);
        <T::Data as driver::DeviceRemoval>::device_remove(data);
        ret?;
        Ok(())
    }
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

