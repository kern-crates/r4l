// SPDX-License-Identifier: GPL-2.0

//! Register map access API.

/// reg_read
pub fn reg_read(
    reg_base: usize, 
    offset: usize, 
) -> u32 {
    let base = reg_base + offset;
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
    unsafe { ::core::ptr::write_volatile(base as _, val) };
    return 0;
}