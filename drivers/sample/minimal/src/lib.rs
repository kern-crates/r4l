// SPDX-License-Identifier: GPL-2.0

//! Rust minimal sample.

#![no_std]
use kernel::prelude::*;

module! {
    type: RustMinimal,
    name: "rust_minimal",
    author: "Rust for Linux Contributors",
    description: "Rust minimal sample",
    license: "GPL",
}

struct RustMinimal {
    numbers: Vec<i32>,
}

impl kernel::Module for RustMinimal {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust minimal sample (init)\n");
        pr_info!("Am I built-in? {}\n", !cfg!(MODULE));

        let mut numbers = Vec::new();

        #[cfg(feature = "no_global_oom_handling")]
        {
            numbers.push(72, GFP_KERNEL)?;
            numbers.push(108, GFP_KERNEL)?;
            numbers.push(200, GFP_KERNEL)?;
        }
        #[cfg(not(feature = "no_global_oom_handling"))]
        {
            numbers.push(72);
            numbers.push(108);
            numbers.push(200);
        }

        Ok(RustMinimal { numbers })
    }
}

impl Drop for RustMinimal {
    fn drop(&mut self) {
        pr_info!("My numbers are {:?}\n", self.numbers);
        pr_info!("Rust minimal sample (exit)\n");
    }
}
