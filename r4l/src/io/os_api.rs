// SPDX-License-Identifier: GPL-2.0

#[cfg(feature = "starry")]
mod os_io_interface {
    use axhal::mem::{phys_to_virt, PhysAddr};

    pub fn ioremap(addr: usize) -> usize {
        phys_to_virt(PhysAddr::from(addr)).as_usize()
    }
}

pub use os_io_interface::*;