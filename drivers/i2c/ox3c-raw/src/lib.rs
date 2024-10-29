// SPDX-License-Identifier: GPL-2.0

//! Rust ofilm_ox3c i2c_camera driver

#![no_std]

use kernel::{of, i2c, prelude::*};


kernel::module_i2c_driver! {
    type: CameraOx3c,
    name: "ofilm_ox3c",
    author: "Heaven Tom",
    description: "Rust ofilm_ox3c driver sample",
    license: "GPL",
}

kernel::module_of_id_table!(OF_MOD_TABLE, OX3C_OF_ID_TABLE);
kernel::define_of_id_table! {OX3C_OF_ID_TABLE, (), [
    (of::DeviceId::Compatible("bst,ofilm_ox3c"), None),
]}

struct CameraOx3c;
impl  i2c::Driver for CameraOx3c {
    type Data = ();

    kernel::driver_of_id_table!(OX3C_OF_ID_TABLE);

    fn probe(client: &mut i2c::I2cClient) -> Result<()> {
        pr_info!("media ofilm_ox3c i2c driver probe start");
        let adpt = client.get_adapter();
        pr_info!("its adapter node name is {}", adpt.get_node().name);

        let functionality = match &adpt.alogrithm.functionality {
            Some(fn_functionality) => fn_functionality(client)?,
            None => panic!("client not have functionality call back"),
        };

        pr_info!("functionality is {} ", functionality);
        pr_info!("media ofilm_ox3c i2c driver probe success");

        Ok(())
    }
}

