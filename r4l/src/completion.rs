// SPDX-License-Identifier: GPL-2.0

//! I2c kernel interface
//!

use crate::sync::Completion as AxCompletion;
use crate::prelude::*;
use alloc::sync::Arc;
use crate::error::Result;

/// Linux completion wrapper
///
/// Wraps the kernel's C `struct completion`.
///
#[repr(transparent)]
pub struct Completion(AxCompletion);

// SAFETY: `Device` only holds a pointer to a C device, which is safe to be used from any thread.
unsafe impl Send for Completion {}

// SAFETY: `Device` only holds a pointer to a C device, references to which are safe to be used
// from any thread.
unsafe impl Sync for Completion {}

/// Creates a [`completion`] initialiser with the given name and a newly-created lock class.
///
/// It uses the name if one is given, otherwise it generates one based on the file name and line
/// number.
#[macro_export]
macro_rules! new_completion {
    ($($name:literal)?) => {
        $crate::completion::Completion::new($crate::optional_name!($($name)?), $crate::static_lock_class!())
    };
}

impl Completion {
    /// Creates a new instance of [`Completion`].
    pub fn new() -> Result<Arc<Self>> {
        Ok(Arc::new(Self(AxCompletion::new())))
    }

    pub fn reinit(&self) {
        self.0.reinit();
    }

    pub fn complete(&self) {
        self.0.complete();
    }

    pub fn wait_for_completion(&self) {
        self.0.wait_for_completion();
    }

    pub fn wait_for_completion_timeout_sec(&self, timeout: usize) -> Result<()> {
        if self.0.wait_for_completion_timeout(timeout as u64) {
            return Err(ETIMEDOUT);
        };
        Ok(())
    }
}