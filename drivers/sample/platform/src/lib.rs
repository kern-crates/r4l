// SPDX-License-Identifier: GPL-2.0

//! Rust platform sample

#![no_std]

use kernel::{module_driver, module_platform_driver, of, platform, prelude::*};

struct PlatformSampleDriver;
module_platform_driver! {
   type: PlatformSampleDriver,
   name: "platform_sample",
   author: "Rust for Linux Contributors",
   description: "Rust minimal sample",
   license: "GPL",
}

// Linux Raw id table
kernel::module_of_id_table!(SAMPLE_MOD_TABLE, SAMPLE_OF_MATCH_TABLE);
// R4L IdArray table
kernel::define_of_id_table! {SAMPLE_OF_MATCH_TABLE, (), [
    (of::DeviceId::Compatible("cicv,sample"),None),
]}

impl platform::Driver for PlatformSampleDriver {
    type Data = ();
    kernel::driver_of_id_table!(SAMPLE_OF_MATCH_TABLE);

    fn probe(pdev: &mut platform::PlatformDevice, _id_info: Option<&Self::IdInfo>) -> Result<()> {
        pr_info!("match is impossible");
        Ok(())
    }
}
