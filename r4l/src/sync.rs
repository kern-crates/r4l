//! Defines the R4L sync.
//!
//! Every OS should provides:
//! - Arc
//! - Mutex
//! - SpinLock

#[cfg(feature = "starry")]
mod sync {
    pub use alloc::sync::Arc;
    pub use axsync::spin::SpinNoIrq;
    pub use axsync::Mutex;
}

pub use sync::*;
