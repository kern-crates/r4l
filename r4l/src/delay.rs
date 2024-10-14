//! I2c delay interface
//!

#[cfg(feature = "starry")]
mod delay {
    pub use axtask::sleep;
}

use delay::*;

/// usleep
pub fn usleep(us: u64) {
    sleep(core::time::Duration::from_nanos(us));
}

