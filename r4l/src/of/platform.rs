// SPDX-License-Identifier: GPL-2.0

use super::DeviceId;
use crate::error::*;
use crate::platform::{platform_device_register, PlatformDevice};
use crate::pr_debug;
use crate::sync::Arc;
use crate::sync::Mutex;
use of::OfNode;

const OF_DEFAULT_BUS_MATCH_TABLE: [&'static str; 4] =
    ["simple-bus", "simple-mfd", "isa", "arm,amba-bus"];

fn of_platform_bus_device_create(node: OfNode<'static>) -> Result {
    if !of::of_device_is_available(node) {
        return Ok(());
    }
    let pdev = Arc::new(Mutex::new(PlatformDevice::new(node)));
    platform_device_register(pdev)?;
    Ok(())
}

pub fn of_platform_default_populate_init() -> Result {
    let bus_node = of::find_compatible_node(&OF_DEFAULT_BUS_MATCH_TABLE);
    for i in bus_node {
        crate::pr_info!("bus node: {:?} ", i.compatible().unwrap().first());
        for c in i.children() {
            of_platform_bus_device_create(c)?;
            match (i.compatible(), c.compatible()) {
                (Some(i), Some(c)) => {
                    crate::pr_info!("create platform {:?} child device {}", i.first(), c.first())
                }
                (Some(i), None) => {
                    crate::pr_info!("create platform {:?} child device none", i.first())
                }
                _ => panic!("no imposible"),
            }
        }
    }
    Ok(())
}
