// SPDX-License-Identifier: GPL-2.0

use super::{i2c_register_driver, I2cClient};
use crate::{
    device::DeviceOps, 
    driver, 
    driver::IdArray, 
    driver::IdTable, 
    error::*, 
    of, 
    prelude::*,
    sync::Arc, sync::Mutex,
};

type PlatformIdTable = &'static [of::DeviceId];

pub struct I2cDriver {
    driver: driver::DeviceDriver,
    pub probe: Option<fn(dev: Arc<Mutex<I2cClient>>) -> Result>,
    remove: Option<fn(dev: &mut I2cClient) -> Result>,
    id_table: Option<PlatformIdTable>,
}

impl Default for I2cDriver {
    fn default() -> Self {
        let mut s = ::core::mem::MaybeUninit::<Self>::uninit();
        unsafe {
            ::core::ptr::write_bytes(s.as_mut_ptr(), 0, 1);
            s.assume_init()
        }
    }
}

impl I2cDriver {
    fn init(
        &mut self,
        probe: fn(dev: Arc<Mutex<I2cClient>>) -> Result,
        remove: fn(dev: &mut I2cClient) -> Result,
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

/// A i2c driver.
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

    /// I2c driver probe.
    ///
    /// Called when a new i2c device is added or discovered.
    /// Implementers should attempt to initialize the device here.
    fn probe(dev: &mut I2cClient) -> Result<Self::Data>;

    /// I2c driver remove.
    ///
    /// Called when a i2c device is removed.
    /// Implementers should prepare the device for complete removal here.
    fn remove(_data: &Self::Data) -> Result {
        Ok(())
    }
}

/// A registration of a i2c driver.
pub type Registration<T> = driver::Registration<Adapter<T>>;

/// An adapter for the registration of i2c drivers.
pub struct Adapter<T: Driver>(T);

impl<T: Driver> driver::DriverOps for Adapter<T> {
    type RegType = Arc<Mutex<I2cDriver>>;

    fn register(
        i2cdrv: &mut Self::RegType,
        name: &'static CStr,
        module: &'static ThisModule,
    ) -> Result {
        i2cdrv.lock().init(
            Self::probe_callback,
            Self::remove_callback,
            T::OF_DEVICE_ID_TABLE,
        );
        i2cdrv.lock().driver.init(name, module)?;
        i2c_register_driver(i2cdrv.clone())?;
        Ok(())
    }

    fn unregister(_i2cdrv: &mut Self::RegType) {}
}

impl<T: Driver> Adapter<T> {

    fn probe_callback(client: Arc<Mutex<I2cClient>>) -> Result {
        let mut client = client.lock();
        let data = T::probe(&mut client)?;
        client.set_drv_data(data.clone());
        Ok(())
    }

    fn remove_callback(pdev: &mut I2cClient) -> Result {
        let data = pdev.get_drv_data::<T::Data>().unwrap();
        T::remove(data)?;
        <T::Data as driver::DeviceRemoval>::device_remove(data);
        Ok(())
    }
}

macro_rules! module_i2c_device {
    ($($f:tt)*) => {
        $crate::module_driver!(<T>, $crate::i2c::Adapter<T>, { $($f)* });
    };
}

/// Declares a kernel module that exposes a single i2c driver.
///
/// # Examples
///
/// ```ignore
/// # use kernel::{i2c, define_i2c_id_table, module_i2c_driver};
/// kernel::module_i2c_id_table!(MOD_TABLE, I2C_CLIENT_I2C_ID_TABLE);
/// kernel::define_i2c_id_table! {I2C_CLIENT_I2C_ID_TABLE, (), [
///     (i2c::DeviceId(b"fpga"), None),
/// ]}
/// struct MyDriver;
/// impl i2c::Driver for MyDriver {
///     kernel::driver_i2c_id_table!(I2C_CLIENT_I2C_ID_TABLE);
///     // [...]
/// #   fn probe(_client: &mut i2c::Client) -> Result {
/// #       Ok(())
/// #   }
/// }
///
/// module_i2c_driver! {
///     type: MyDriver,
///     name: "module_name",
///     author: "Author name",
///     license: "GPL",
/// }
/// ```

#[macro_export]
macro_rules! module_i2c_driver {
    ($($f:tt)*) => {
        $crate::module_driver!(<T>, $crate::i2c::Adapter<T>, { $($f)* });
    };
}
