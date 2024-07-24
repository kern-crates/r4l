// SPDX-License-Identifier: GPL-2.0

#[cfg(feature = "starry")]
mod os_irq_interface {
    use axhal::irq::register_handler;

    pub type IrqHandler = axhal::irq::IrqHandler;
    pub fn request_threaded_irq(irq: u32, handler: IrqHandler) {
        register_handler(irq as usize, handler);
    }
}

pub use os_irq_interface::*;
