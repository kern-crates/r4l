// SPDX-License-Identifier: GPL-2.0

//! Network PHY device
//!
//! Linux about net phy driver general struct defines
//! 
//! Where R4L originally used binding, it needs to
//! be replaced with OS's own platform implementation
//!
//! Each OS should implement this type
//! pub struct  PhyDevice {
//!   pub phy_id:u32,
//!   pub state: DeviceState,
//!   pub speed: i32,
//!   pub duplex: DuplexMode,
//! }
//!
//! impl PhyDeviceOps for PhyDevice; 
//!

mod phy_drv;
mod phy_dev;
pub use phy_dev::PhyDevice;
pub use phy_drv::PhyDriver;


use crate::{error::*, prelude::*};
use core::marker::PhantomData;
use bitflags::bitflags;

bitflags! {
    /// To determine what I2C functionality is present
    #[repr(transparent)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct PhyDriverFlags: u32 {
        /// PHY is internal.
        const IS_INTERNAL = 0x00000001;
        /// PHY needs to be reset after the refclk is enabled.
        const RST_AFTER_CLK_EN = 0x00000002;
        /// Polling is used to detect PHY status changes.
        const POLL_CABLE_TEST = 0x00000004;
        /// Don't suspend.
        const ALWAYS_CALL_SUSPEND = 0x00000008;
    }
}

pub trait PhyDeviceOps {
    /// Gets the id of the PHY.
    fn phy_id(&self) -> u32;
    /// Gets the state of PHY state machine states.
    fn state(&self) -> DeviceState;
    /// Gets the current link state.
    ///
    /// It returns true if the link is up.
    fn is_link_up(&self) -> bool;
    /// Gets the current auto-negotiation configuration.
    ///
    /// It returns true if auto-negotiation is enabled.
    fn is_autoneg_enabled(&self) -> bool;
    /// Gets the current auto-negotiation state.
    ///
    /// It returns true if auto-negotiation is completed.
    fn is_autoneg_completed(&self) -> bool;
    /// Sets the speed of the PHY.
    fn set_speed(&mut self, speed: u32);
    /// Sets duplex mode.
    fn set_duplex(&mut self, mode: DuplexMode);
    /// Reads a given C22 PHY register.
    // This function reads a hardware register and updates the stats so takes `&mut self`.
    fn read(&mut self, regnum: u16) -> Result<u16> ;
    /// Writes a given C22 PHY register.
    fn write(&mut self, regnum: u16, val: u16) -> Result;
    /// Reads a paged register.
    fn read_paged(&mut self, page: u16, regnum: u16) -> Result<u16>;
    /// Resolves the advertisements into PHY settings.
    fn resolve_aneg_linkmode(&mut self);
    /// Executes software reset the PHY via `BMCR_RESET` bit.
    fn genphy_soft_reset(&mut self) -> Result;
    /// Initializes the PHY.
    fn init_hw(&mut self) -> Result;
    /// Starts auto-negotiation.
    fn start_aneg(&mut self) -> Result;
    /// Resumes the PHY via `BMCR_PDOWN` bit.
    fn genphy_resume(&mut self) -> Result;
    /// Suspends the PHY via `BMCR_PDOWN` bit.
    fn genphy_suspend(&mut self) -> Result;
    /// Checks the link status and updates current link state.
    fn genphy_read_status(&mut self) -> Result<u16>;
    /// Updates the link status.
    fn genphy_update_link(&mut self) -> Result;
    /// Reads link partner ability.
    fn genphy_read_lpa(&mut self) -> Result;
    /// Reads PHY abilities.
    fn genphy_read_abilities(&mut self) -> Result;
}

/// PHY state machine states.
///
/// Corresponds to the kernel's [`enum phy_state`].
///
/// Some of PHY drivers access to the state of PHY's software state machine.
///
/// [`enum phy_state`]: srctree/include/linux/phy.h
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DeviceState {
    /// PHY device and driver are not ready for anything.
    Down,
    /// PHY is ready to send and receive packets.
    Ready,
    /// PHY is up, but no polling or interrupts are done.
    Halted,
    /// PHY is up, but is in an error state.
    Error,
    /// PHY and attached device are ready to do work.
    Up,
    /// PHY is currently running.
    Running,
    /// PHY is up, but not currently plugged in.
    NoLink,
    /// PHY is performing a cable test.
    CableTest,
}

/// An instance of a PHY device.
///
/// Linux Wraps the kernel's [`struct phy_device`].
/// On rust os no need to wrapper
pub type Device = PhyDevice;

/// An adapter for the registration of a PHY driver.
struct Adapter<T: Driver> {
    _p: PhantomData<T>,
}
/// Driver structure for a particular PHY type.
impl<T: Driver> Adapter<T> {
    fn soft_reset_callback(dev: &mut Device) -> Result {
        T::soft_reset(dev)
    }

    fn get_features_callback(dev: &mut Device) -> Result {
        T::get_features(dev)
    }

    fn suspend_callback(phydev: &mut Device) -> Result {
        T::suspend(phydev)
    }

    fn resume_callback(phydev: &mut Device) -> Result {
        T::resume(phydev)
    }

    fn config_aneg_callback(phydev: &mut Device) -> Result {
        T::config_aneg(phydev)
    }

    fn read_status_callback(phydev: &mut Device) -> Result<u16> {
        T::read_status(phydev)
    }

    fn match_phy_device_callback(phydev: &Device)-> bool {
        T::match_phy_device(phydev)
    }

    fn read_mmd_callback(
        dev: &mut Device,
        devnum: u8,
        regnum: u16,
    ) -> Result<u16> {
        T::read_mmd(dev, devnum, regnum)
    }

    /// # Safety
    ///
    /// `phydev` must be passed by the corresponding callback in `phy_driver`.
    fn write_mmd_callback(
        dev: &mut Device,
        devnum: u8,
        regnum: u16,
        val: u16,
    ) -> Result {
        T::write_mmd(dev, devnum, regnum, val)
    }

    fn link_change_notify_callback(phydev: &mut Device) {
        T::link_change_notify(phydev)
    }
}

/// A mode of Ethernet communication.
///
/// PHY drivers get duplex information from hardware and update the current state.
pub enum DuplexMode {
    /// PHY is in full-duplex mode.
    Full,
    /// PHY is in half-duplex mode.
    Half,
    /// PHY is in unknown duplex mode.
    Unknown,
}

///
/// Wraps the kernel's [`struct phy_driver`].
/// This is used to register a driver for a particular PHY type with the kernel.
///
/// # Invariants
///
/// `self.0` is always in a valid state.
///
/// [`struct phy_driver`]: srctree/include/linux/phy.h
#[repr(transparent)]
pub struct DriverVTable(PhyDriver);

// SAFETY: `DriverVTable` doesn't expose any &self method to access internal data, so it's safe to
// share `&DriverVTable` across execution context boundries.
unsafe impl Sync for DriverVTable {}

/// Creates a [`DriverVTable`] instance from [`Driver`].
///
/// This is used by [`module_phy_driver`] macro to create a static array of `phy_driver`.
///
/// [`module_phy_driver`]: crate::module_phy_driver
pub const fn create_phy_driver<T: Driver>() -> DriverVTable {
    // INVARIANT: All the fields of `struct phy_driver` are initialized properly.
    DriverVTable(PhyDriver{
        name: T::NAME,
        flags: T::FLAGS,
        deviceid: T::PHY_DEVICE_ID,
        soft_reset: if T::HAS_SOFT_RESET {
            Some(Adapter::<T>::soft_reset_callback)
        } else {
            None
        },
        get_features: if T::HAS_GET_FEATURES {
            Some(Adapter::<T>::get_features_callback)
        } else {
            None
        },
        match_phy_device: if T::HAS_MATCH_PHY_DEVICE {
            Some(Adapter::<T>::match_phy_device_callback)
        } else {
            None
        },
        suspend: if T::HAS_SUSPEND {
            Some(Adapter::<T>::suspend_callback)
        } else {
            None
        },
        resume: if T::HAS_RESUME {
            Some(Adapter::<T>::resume_callback)
        } else {
            None
        },
        config_aneg: if T::HAS_CONFIG_ANEG {
            Some(Adapter::<T>::config_aneg_callback)
        } else {
            None
        },
        read_status: if T::HAS_READ_STATUS {
           Some(Adapter::<T>::read_status_callback)
        } else {
            None
        },
        read_mmd: if T::HAS_READ_MMD {
            Some(Adapter::<T>::read_mmd_callback)
        } else {
            None
        },
        write_mmd: if T::HAS_WRITE_MMD {
            Some(Adapter::<T>::write_mmd_callback)
        } else {
            None
        },
        link_change_notify: if T::HAS_LINK_CHANGE_NOTIFY {
            Some(Adapter::<T>::link_change_notify_callback)
        } else {
            None
        },
        ..unsafe { core::mem::MaybeUninit::<bindings::phy_driver>::zeroed().assume_init()}
    })
}

/// Driver implementation for a particular PHY type.
///
/// This trait is used to create a [`DriverVTable`].
#[vtable]
pub trait Driver {
    /// Defines certain other features this PHY supports.
    /// It is a combination of the flags in the [`flags`] module.
    const FLAGS: PhyDriverFlags =  PhyDriverFlags::empty();

    /// The friendly name of this PHY type.
    const NAME: &'static CStr;

    /// This driver only works for PHYs with IDs which match this field.
    /// The default id and mask are zero.
    const PHY_DEVICE_ID: DeviceId = DeviceId::new_with_custom_mask(0, 0);

    /// Issues a PHY software reset.
    fn soft_reset(_dev: &mut Device) -> Result {
        kernel::build_error(VTABLE_DEFAULT_ERROR)
    }

    /// Probes the hardware to determine what abilities it has.
    fn get_features(_dev: &mut Device) -> Result {
        kernel::build_error(VTABLE_DEFAULT_ERROR)
    }

    /// Returns true if this is a suitable driver for the given phydev.
    /// If not implemented, matching is based on [`Driver::PHY_DEVICE_ID`].
    fn match_phy_device(_dev: &Device) -> bool {
        false
    }

    /// Configures the advertisement and resets auto-negotiation
    /// if auto-negotiation is enabled.
    fn config_aneg(_dev: &mut Device) -> Result {
        kernel::build_error(VTABLE_DEFAULT_ERROR)
    }

    /// Determines the negotiated speed and duplex.
    fn read_status(_dev: &mut Device) -> Result<u16> {
        kernel::build_error(VTABLE_DEFAULT_ERROR)
    }

    /// Suspends the hardware, saving state if needed.
    fn suspend(_dev: &mut Device) -> Result {
        kernel::build_error(VTABLE_DEFAULT_ERROR)
    }

    /// Resumes the hardware, restoring state if needed.
    fn resume(_dev: &mut Device) -> Result {
        kernel::build_error(VTABLE_DEFAULT_ERROR)
    }

    /// Overrides the default MMD read function for reading a MMD register.
    fn read_mmd(_dev: &mut Device, _devnum: u8, _regnum: u16) -> Result<u16> {
        kernel::build_error(VTABLE_DEFAULT_ERROR)
    }

    /// Overrides the default MMD write function for writing a MMD register.
    fn write_mmd(_dev: &mut Device, _devnum: u8, _regnum: u16, _val: u16) -> Result {
        kernel::build_error(VTABLE_DEFAULT_ERROR)
    }

    /// Callback for notification of link change.
    fn link_change_notify(_dev: &mut Device) {}
}

/// Registration structure for PHY drivers.
///
/// Registers [`DriverVTable`] instances with the kernel. They will be unregistered when dropped.
///
/// # Invariants
///
/// The `drivers` slice are currently registered to the kernel via `phy_drivers_register`.
pub struct Registration {
    drivers: core::pin::Pin<&'static mut [DriverVTable]>,
}

// SAFETY: The only action allowed in a `Registration` instance is dropping it, which is safe to do
// from any thread because `phy_drivers_unregister` can be called from any thread context.
unsafe impl Send for Registration {}

impl Registration {
    /// Registers a PHY driver.
    pub fn register(
        _module: &'static crate::ThisModule,
        drivers: core::pin::Pin<&'static mut [DriverVTable]>,
    ) -> Result<Self> {
        if drivers.is_empty() {
            return Err(code::EINVAL);
        }
        //to_result(unsafe {
        //     bindings::phy_drivers_register(drivers[0].0.get(), drivers.len().try_into()?, module.0)
        // })?;
        // INVARIANT: The `drivers` slice is successfully registered to the kernel via `phy_drivers_register`.
        Ok(Registration { drivers })
    }
}

impl Drop for Registration {
    fn drop(&mut self) {
        // SAFETY: The type invariants guarantee that `self.drivers` is valid.
        // So it's just an FFI call.
        /*
        unsafe {
            bindings::phy_drivers_unregister(self.drivers[0].0.get(), self.drivers.len() as i32)
        };
        */
    }
}

/// An identifier for PHY devices on an MDIO/MII bus.
///
/// Represents the kernel's `struct mdio_device_id`. This is used to find an appropriate
/// PHY driver.
pub struct DeviceId {
    id: u32,
    mask: DeviceMask,
}

impl DeviceId {
    /// Creates a new instance with the exact match mask.
    pub const fn new_with_exact_mask(id: u32) -> Self {
        DeviceId {
            id,
            mask: DeviceMask::Exact,
        }
    }

    /// Creates a new instance with the model match mask.
    pub const fn new_with_model_mask(id: u32) -> Self {
        DeviceId {
            id,
            mask: DeviceMask::Model,
        }
    }

    /// Creates a new instance with the vendor match mask.
    pub const fn new_with_vendor_mask(id: u32) -> Self {
        DeviceId {
            id,
            mask: DeviceMask::Vendor,
        }
    }

    /// Creates a new instance with a custom match mask.
    pub const fn new_with_custom_mask(id: u32, mask: u32) -> Self {
        DeviceId {
            id,
            mask: DeviceMask::Custom(mask),
        }
    }

    /// Creates a new instance from [`Driver`].
    pub const fn new_with_driver<T: Driver>() -> Self {
        T::PHY_DEVICE_ID
    }

    /// Get a `mask` as u32.
    pub const fn mask_as_int(&self) -> u32 {
        self.mask.as_int()
    }
}

enum DeviceMask {
    Exact,
    Model,
    Vendor,
    Custom(u32),
}

impl DeviceMask {
    const MASK_EXACT: u32 = !0;
    const MASK_MODEL: u32 = !0 << 4;
    const MASK_VENDOR: u32 = !0 << 10;

    const fn as_int(&self) -> u32 {
        match self {
            DeviceMask::Exact => Self::MASK_EXACT,
            DeviceMask::Model => Self::MASK_MODEL,
            DeviceMask::Vendor => Self::MASK_VENDOR,
            DeviceMask::Custom(mask) => *mask,
        }
    }
}

/// Declares a kernel module for PHYs drivers.
///
/// This creates a static array of kernel's `struct phy_driver` and registers it.
/// This also corresponds to the kernel's `MODULE_DEVICE_TABLE` macro, which embeds the information
/// for module loading into the module binary file. Every driver needs an entry in `device_table`.
///
/// # Examples
///
/// ```
/// # mod module_phy_driver_sample {
/// use kernel::c_str;
/// use kernel::net::phy::{self, DeviceId};
/// use kernel::prelude::*;
///
/// kernel::module_phy_driver! {
///     drivers: [PhySample],
///     device_table: [
///         DeviceId::new_with_driver::<PhySample>()
///     ],
///     name: "rust_sample_phy",
///     author: "Rust for Linux Contributors",
///     description: "Rust sample PHYs driver",
///     license: "GPL",
/// }
///
/// struct PhySample;
///
/// #[vtable]
/// impl phy::Driver for PhySample {
///     const NAME: &'static CStr = c_str!("PhySample");
///     const PHY_DEVICE_ID: phy::DeviceId = phy::DeviceId::new_with_exact_mask(0x00000001);
/// }
/// # }
/// ```
///
/// This expands to the following code:
///
/// ```ignore
/// use kernel::c_str;
/// use kernel::net::phy::{self, DeviceId};
/// use kernel::prelude::*;
///
/// struct Module {
///     _reg: ::kernel::net::phy::Registration,
/// }
///
/// module! {
///     type: Module,
///     name: "rust_sample_phy",
///     author: "Rust for Linux Contributors",
///     description: "Rust sample PHYs driver",
///     license: "GPL",
/// }
///
/// struct PhySample;
///
/// #[vtable]
/// impl phy::Driver for PhySample {
///     const NAME: &'static CStr = c_str!("PhySample");
///     const PHY_DEVICE_ID: phy::DeviceId = phy::DeviceId::new_with_exact_mask(0x00000001);
/// }
///
/// const _: () = {
///     static mut DRIVERS: [::kernel::net::phy::DriverVTable; 1] =
///         [::kernel::net::phy::create_phy_driver::<PhySample>()];
///
///     impl ::kernel::Module for Module {
///         fn init(module: &'static ThisModule) -> Result<Self> {
///             let drivers = unsafe { &'static mut DRIVERS };
///             let mut reg = ::kernel::net::phy::Registration::register(
///                 module,
///                 drivers,
///             )?;
///             Ok(Module { _reg: reg })
///         }
///     }
/// };
///
#[macro_export]
macro_rules! module_phy_driver {
    (@replace_expr $_t:tt $sub:expr) => {$sub};

    (@count_devices $($x:expr),*) => {
        0usize $(+ $crate::module_phy_driver!(@replace_expr $x 1usize))*
    };

    (drivers: [$($driver:ident),+ $(,)?], device_table: [$($dev:expr),+ $(,)?], $($f:tt)*) => {
        struct Module {
            _reg: $crate::net::phy::Registration,
        }

        $crate::prelude::module! {
            type: Module,
            $($f)*
        }

        const _: () = {
            static mut DRIVERS: [$crate::net::phy::DriverVTable;
                $crate::module_phy_driver!(@count_devices $($driver),+)] =
                [$($crate::net::phy::create_phy_driver::<$driver>()),+];

            impl $crate::Module for Module {
                fn init(module: &'static ThisModule) -> Result<Self> {
                    let drivers = unsafe { &mut DRIVERS};
                    let mut reg = $crate::net::phy::Registration::register(
                        module,
                         ::core::pin::Pin::static_mut(drivers),
                    )?;
                    Ok(Module { _reg: reg })
                }
            }
        };
    }
}


