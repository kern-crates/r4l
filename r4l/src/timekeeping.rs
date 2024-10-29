// SPDX-License-Identifier: GPL-2.0

//! I2c kernel interface
//!

#[cfg(feature = "arceos")]
mod timekeeping {
    pub use axhal::time::monotonic_time_nanos;

    /// Get Ktime
    pub fn ktime_get() -> u64 {
        monotonic_time_nanos()
    }
}

pub use timekeeping::*;
use crate::{
    prelude::*,
    delay,
    error::Result,
};

/// One usec to nsec
pub const NSEC_PER_USEC: u64 = 1000;

/// ktime add us
pub fn time_add_us(us: u64) -> u64 {
    ktime_get() + us * NSEC_PER_USEC
}

/// current time
pub fn current_time() -> u64 {
    ktime_get()
}

/// Poll until a condition is met or a timeout occurs
pub fn read_poll_timeout<T, F: Fn() -> T, C: Fn(T) -> bool>(
    read_op: F,
    cond: C,
    sleep_us: u64,
    timeout_us: u64,
    sleep_before: bool,
) -> Result<()> {
    let timeout: u64 = time_add_us(timeout_us);

    if sleep_us != 0 && sleep_before {
        delay::usleep(sleep_us);
    }

    let ret = loop {
        if cond(read_op()) {
            return Ok(());
        }

        if timeout_us != 0 && current_time() > timeout {
            break read_op();
        }

        if sleep_us > 0 {
            delay::usleep(sleep_us);
        }
    };

    if cond(ret) {
        return Ok(());
    } else {
        return Err(ETIMEDOUT);
    }
}