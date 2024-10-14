// SPDX-License-Identifier: GPL-2.0

//! Register map access API.

use crate::pr_info;

/// reg_read
pub fn reg_read(
    reg_base: usize, 
    offset: usize, 
) -> u32 {
    let base = reg_base + offset;
    pr_info!("reg_read: reg_base is {:#x}, offset is {:#x}, reg_addr is {:#x}", reg_base, offset, base);
    let val = unsafe {::core::ptr::read_volatile(base as _)};
    val
}

/// reg_write
pub fn reg_write(
    reg_base: usize, 
    offset: usize, 
    val: u32,
) -> u32 {
    let base = reg_base + offset ;
    pr_info!("reg_write: reg_base is {:#x}, offset is {:#x}, reg_addr is {:#x}, val is {val:#}", reg_base, offset, base);
    unsafe { ::core::ptr::write_volatile(base as _, val) };
    return 0;
}