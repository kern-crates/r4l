// SPDX-License-Identifier: GPL-2.0

//! Rust dw_apb_i2c

#![no_std]

use kernel::{
    driver,
    irq,
    device::Device,
    module_platform_driver, of, platform,
    sync::Arc,
    prelude::*,
};

module_platform_driver! {
      type: DwI2cDriver,
      name: "i2c_designware",
      license: "GPL",
      initcall: "subsys",
}

// Linux Raw id table
kernel::module_of_id_table!(DW_I2C_MOD_TABLE, DW_I2C_OF_MATCH_TABLE);
// R4L IdArray table
kernel::define_of_id_table! {DW_I2C_OF_MATCH_TABLE, (), [
    (of::DeviceId::Compatible("snps,designware-i2c"),None),
]}

struct DwI2cData();

impl driver::DeviceRemoval for DwI2cData {
    fn device_remove(&self) {
        pr_info!("unimplement DwI2cData Remove");
    }
}

struct DwI2cIrqHandler;
impl irq::Handler for DwI2cIrqHandler {
    type Data = i32;

    fn handle_irq(data: &i32) -> irq::Return {
        pr_info!("handled i2c irq get data {} ", data);
        irq::Return::Handled
    }
}

struct DwI2cDriver;
impl platform::Driver for DwI2cDriver {
    type Data = Arc<DwI2cData>;
    // Linux Raw id table
    kernel::driver_of_id_table!(DW_I2C_OF_MATCH_TABLE);

    fn probe(pdev: &mut platform::Device,_id_info: Option<&Self::IdInfo>,
    ) -> Result<Self::Data> {
        let irq = pdev.irq_resource(0)?;
        pr_err!("enter i2c platform probe func, get irq {}",irq);
        Ok(Arc::new(DwI2cData()))
    }
}
