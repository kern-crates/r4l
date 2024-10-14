//! Defines the R4L sync.
//!
//! Every OS should provides:
//! - Arc
//! - Mutex
//! - SpinLock

#[cfg(feature = "starry")]
mod sync {
    pub use alloc::sync::Arc;
    pub use axsync::spin::{self, SpinNoIrq, SpinNoPreempt};
    pub use axsync::Mutex;
    // pub use axsync::Completion;

    pub type  SpinLock<T> = SpinNoPreempt<T>;
}

pub use sync::*;

#[macro_export]
macro_rules! new_spinlock {
    ($inner:expr $(, $name:literal)? $(,)?) => {
        $crate::sync::SpinLock::new($inner)
    };
}