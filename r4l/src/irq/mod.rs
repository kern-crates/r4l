// SPDX-License-Identifier: GPL-2.0

//! Interrupts.
//!
//! See <https://www.kernel.org/doc/Documentation/core-api/genericirq.rst>.
//!
//! Compatible with r4l's irq module interface
//!

mod flags;
pub use flags::*;

mod os_api;
pub use os_api::*;

use crate::{
    error::Result,
    str::CString,
};

use crate::prelude::*;
use crate::sync::Mutex;
use core::fmt;
use core::any::Any;

/// The return value from interrupt handlers.
pub enum Return {
    /// The interrupt was not from this device or was not handled.
    None,
    /// The interrupt was handled by this device.
    Handled,
    /// The handler wants the handler thread to wake up.
    /// Maybe os not support this
    WakeThread,
}

struct InternalRegistration {
    irq: u32,
    name: CString,
}

impl  InternalRegistration {
    /// Registers a new irq handler.
    fn try_new(
        irq: u32,
        handler: IrqHandler,
        _thread_fn: Option<IrqHandler>,
        flags: usize,
        name: fmt::Arguments<'_>,
    ) -> Result<Self> {
        let name = CString::try_from_fmt(name)?;
        // setup os irq handler
        request_threaded_irq(irq, handler);
        Ok(Self {
            irq,
            name,
        })
    }
}

impl Drop for InternalRegistration {
    fn drop(&mut self) {
        // Unregister irq handler.
    }
}

/// An irq handler.
pub trait Handler {
    /// The context data associated with and made available to the handler.
    type Data = ();

    /// Called from interrupt context when the irq happens.
    fn handle_irq(data: &Self::Data) -> Return;
}

/// The registration of an interrupt handler.
///
/// # Examples
///
/// The following is an example of a regular handler with a boxed `u32` as data.
///
/// ```
/// # use kernel::prelude::*;
/// use kernel::irq;
///
/// struct Example;
///
/// impl irq::Handler for Example {
///     type Data = Box<u32>;
///
///     fn handle_irq(_data: &u32) -> irq::Return {
///         irq::Return::None
///     }
/// }
///
/// fn request_irq(irq: u32, data: Box<u32>) -> Result<irq::Registration<Example>> {
///     irq::Registration::try_new(irq, data, irq::flags::SHARED, fmt!("example_{irq}"))
/// }
/// ```
pub struct Registration(InternalRegistration);

unsafe impl Send for Registration {}
unsafe impl Sync for Registration {}

struct IrqData {
    data: Box<dyn Any>,
    irq: u32
}

unsafe impl Send for IrqData {}
unsafe impl Sync for IrqData {}

static IRQ_DATA_ARRAY: Mutex<Vec<IrqData>> = Mutex::new(Vec::new());

impl Registration {
    /// Registers a new irq handler.
    ///
    /// The valid values of `flags` come from the [`flags`] module.
    pub fn try_new<H: Handler> (
        irq: u32,
        data: H::Data,
        flags: usize,
        name: fmt::Arguments<'_>,
    ) -> Result<Self>  where <H as Handler>::Data: 'static {
        IRQ_DATA_ARRAY.lock().push(IrqData{data: Box::new(data), irq});
        Ok(Self(InternalRegistration::try_new(irq, Self::handler::<H>, None, flags, name)?))
    }

    #[cfg(feature = "starry")]
    fn handler<H: Handler> (irq:u32) where <H as Handler>::Data: 'static {
        let lock = IRQ_DATA_ARRAY.lock();
        let irq_data = lock.iter().find(|x|x.irq == irq).unwrap();
        H::handle_irq(irq_data.data.downcast_ref::<H::Data>().unwrap());
    }
}

impl Drop for Registration {
    fn drop(&mut self) {
        IRQ_DATA_ARRAY.lock().retain(|x:&IrqData| x.irq != self.0.irq);
    }
}
